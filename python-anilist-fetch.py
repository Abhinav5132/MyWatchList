import requests
import json
import os
import time

JSONL_FILE = "anilist_data.jsonl"
FINAL_JSON_FILE = "anilist_data.json"
BATCH_SIZE = 50
MAX_PAGES = 800  # or adjust as needed

def fetch_page(page, per_page=BATCH_SIZE):
    query = '''
    query ($page: Int, $perPage: Int) {
        Page(page: $page, perPage: $perPage) {
            media(type: ANIME, sort: [POPULARITY_DESC]) {
                id
                title {
                    romaji
                }
                format
                episodes
                status
                season
                seasonYear
                coverImage {
                    extraLarge
                }
                duration
                popularity
                averageScore
                synonyms
                tags {
                    name
                }
                studios {
                    nodes {
                        name
                    }
                }
                relations {
                    nodes {
                        title {
                            romaji
                        }
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
            json.dump(item, f)
            f.write("\n")

def convert_jsonl_to_json(jsonl_path, output_path):
    with open(jsonl_path, "r", encoding="utf-8") as f:
        items = [json.loads(line) for line in f if line.strip()]

    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(items, f, indent=2)

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
            time.sleep(1.5)  # polite delay to avoid rate limits
        except Exception as e:
            print(f"Error on page {page}: {e}")
            break

    print("Converting to JSON...")
    convert_jsonl_to_json(JSONL_FILE, FINAL_JSON_FILE)
    print(f"Done. Final data saved to {FINAL_JSON_FILE}")

if __name__ == "__main__":
    main()
