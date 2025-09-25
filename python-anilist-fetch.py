import requests
import json
import os
import time
import re
from html import unescape

JSONL_FILE = "anilist_data.jsonl"
FINAL_JSON_FILE = "anilist_data.json"
BATCH_SIZE = 50
MAX_PAGES = 800  # Adjust as needed


def clean_description(description):
    if not description:
        return ""
    # Remove HTML tags
    description = re.sub(r"<.*?>", "", description)
    return unescape(description).strip()


def build_trailer_url(trailer):
    if trailer and trailer.get("site") and trailer.get("id"):
        site = trailer["site"].lower()
        trailer_id = trailer["id"]
        if site == "youtube":
            return f"https://www.youtube.com/watch?v={trailer_id}"
        elif site == "dailymotion":
            return f"https://www.dailymotion.com/video/{trailer_id}"
    return None


def format_date(date_dict):
    if not date_dict:
        return None
    year = date_dict.get("year")
    month = date_dict.get("month")
    day = date_dict.get("day")
    if year and month and day:
        return f"{year:04d}-{month:02d}-{day:02d}"
    return None


def simplify_entry(media):
    return {
        "id": media["id"],
        "titleRomaji": media["title"]["romaji"],
        "titleEnglish": media["title"]["english"],
        "description": clean_description(media.get("description")),
        "format": media.get("format"),
        "episodes": media.get("episodes"),
        "status": media.get("status"),
        "season": media.get("season"),
        "seasonYear": media.get("seasonYear"),
        "startDate": format_date(media.get("startDate")),
        "endDate": format_date(media.get("endDate")),
        "thumbnailImage": media["coverImage"]["medium"] if media.get("coverImage") else None,
        "coverImage": media["coverImage"]["extraLarge"] if media.get("coverImage") else None,
        "bannerImage": media.get("bannerImage"),
        "duration": media.get("duration"), 
        "popularity": media.get("popularity"),
        "averageScore": media.get("averageScore"),
        "synonyms": media.get("synonyms", []),
        "tags": [tag["name"] for tag in media.get("tags", [])],
        "genres": media.get("genres", []),
        "studios": [studio["name"] for studio in media["studios"]["nodes"]],
        "relations": [
            {
                "title": rel["node"]["title"]["romaji"],
                "type": rel["relationType"]
            }
            for rel in media.get("relations", {}).get("edges", [])
        ],
        "characters": [
            {
                "name": char["node"]["name"]["full"],
                "role": char["role"],
                "image": char["node"].get("image", {}).get("medium"),
                "voiceActors": [va["name"]["full"] for va in char.get("voiceActors", [])]
            }
            for char in media.get("characters", {}).get("edges", [])
        ],
        "trailer": build_trailer_url(media.get("trailer")),
        "recommendations": [
            {
                "title": rec["mediaRecommendation"]["title"]["romaji"],
                "rating": rec["rating"]
            }
            for rec in media.get("recommendations", {}).get("nodes", [])
            if rec.get("mediaRecommendation")
        ],

        "nextAiringEpisode": (
            media["airingSchedule"]["nodes"][0]
            if media.get("airingSchedule") and media["airingSchedule"].get("nodes")
            else None
            ),
    }


def fetch_page(page, per_page=BATCH_SIZE):
    query = '''
    query ($page: Int, $perPage: Int) {
        Page(page: $page, perPage: $perPage) {
            media(type: ANIME, sort: [POPULARITY_DESC]) {
                id
                title {
                    romaji
                    english
                }
                description
                format
                episodes
                status
                season
                seasonYear
                startDate {
                    year
                    month
                    day
                }
                endDate {
                    year
                    month
                    day
                }
                coverImage {
                    extraLarge
                    medium
                }
                bannerImage
                duration
                popularity
                averageScore
                synonyms
                genres
                tags {
                    name
                }
                studios {
                    nodes {
                        name
                    }
                }
                relations {
                    edges {
                        relationType
                        node {
                            title {
                                romaji
                            }
                        }
                    }
                }
                characters(perPage: 10, sort: [ROLE, RELEVANCE]) {
                    edges {
                        role
                        node {
                            name {
                                full
                            }
                            image {
                                medium
                            }
                        }
                        voiceActors {
                            name {
                                full
                            }
                        }
                    }
                }
                trailer {
                    site
                    id
                }
                recommendations(perPage: 10, sort: [RATING_DESC]) {
                    nodes {
                        mediaRecommendation {
                            title {
                                romaji
                            }
                        }
                        rating
                    }
                }

                airingSchedule(notYetAired: true, perPage: 1) {
                    nodes {
                        episode
                        airingAt
                    }
                }
            }
        }
    }
    '''

    variables = {
        "page": page,
        "perPage": per_page
    }

    response = requests.post("https://graphql.anilist.co", json={"query": query, "variables": variables})
    response.raise_for_status()
    return response.json()["data"]["Page"]["media"]


def write_jsonl(data, path):
    with open(path, "a", encoding="utf-8") as f:
        for item in data:
            simplified = simplify_entry(item)
            json.dump(simplified, f, ensure_ascii=False)
            f.write("\n")


def convert_jsonl_to_json(jsonl_path, output_path):
    with open(jsonl_path, "r", encoding="utf-8") as f:
        items = [json.loads(line) for line in f if line.strip()]

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(items, f, indent=2, ensure_ascii=False)

    os.remove(jsonl_path)


def main():
    if os.path.exists(JSONL_FILE):
        os.remove(JSONL_FILE)

    for page in range(1, MAX_PAGES + 1):
        print(f"Fetching page {page}...")
        try:
            data = fetch_page(page)
            if not data:
                print("No more data.")
                break
            write_jsonl(data, JSONL_FILE)
            time.sleep(2)  # polite delay
        except Exception as e:
            print(f"Error on page {page}: {e}")
            break

    print("Converting to JSON...")
    convert_jsonl_to_json(JSONL_FILE, FINAL_JSON_FILE)
    print(f"Done. Final data saved to {FINAL_JSON_FILE}")


if __name__ == "__main__":
    main()
