#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::backend::{
        add_to_list::{IfRanked, IsRanked, get_if_ranked},
        setup_db,
        verification_service::{MockVerificationService, TokenVerifier},
    };

    use actix_web::{App, test, web::Data};
    use sqlx::{Executor, Pool, Sqlite};

    async fn setup_user(pool: &Data<Pool<Sqlite>>) -> anyhow::Result<bool> {
        match pool.execute("
            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token)
            VALUES ('test','test@test.com','pwd','pfp',
            '69',
            '69');
        ").await {
            Ok(_) => Ok(true),
            Err(e) => {
                dbg!(e);
                Ok(false)
            }
        }
    }

    async fn clean_db(pool: &Data<Pool<Sqlite>>) {
        let _ = pool.execute("DELETE FROM watch_list_anime;").await;
        let _ = pool.execute("DELETE FROM watch_list;").await;
        let _ = pool.execute("DELETE FROM user;").await;

        // Reset AUTOINCREMENT counters (important for IDs = 1)
        let _ = pool.execute("DELETE FROM sqlite_sequence;").await;
    }

    #[actix_web::test]
    async fn test_get_if_ranked() {
        let pool = setup_db().await;
        clean_db(&pool).await;
        match setup_user(&pool).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e}");
            }
        }
        match pool.execute("
            INSERT INTO watch_list(name, user_id, privacy_type, is_ranked, list_image, is_user_image)
            VALUES ('TestList', 1, 'public', 1, '', 0);

            INSERT INTO watch_list_anime(user_id, list_id, anime_id, rank)
            VALUES (1,1,1,10);
        ").await {
            Ok(_) => {},
            Err(e) => {
                dbg!(e);
                panic!();
            }
        }

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: true,
        });
        let app = test::init_service(
            App::new()
                .app_data(pool)
                .app_data(Data::from(verifier.clone()))
                .service(get_if_ranked),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/get-if-ranked")
            .insert_header(("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjEsImV4cCI6MTc2Njk1ODAxNCwiaWF0IjoxNzY0MzY2MDE0fQ.CV5We6lwAkSl8u0LaorQxJDcXmfo3Pra_0Ud64BgUn4"))
            .set_json(&IfRanked{
                user_id: 1,
                list_id: 1,
            }).to_request();
        let resp = test::call_service(&app, req).await;
        dbg!(resp.status());
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;

        let result: IsRanked = serde_json::from_slice(&body).expect("Failed to convert to json");
        assert_eq!(result.is_ranked, 1);
        assert_eq!(result.last_rank, 10);
    }
}
