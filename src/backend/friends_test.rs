#[cfg(test)]
mod tests{
    use std::sync::Arc;
    use reqwest::StatusCode;
    use sqlx::Row;
    use crate::{backend::{friends::{FriendRequest, send_friend_request}, setup_db, verification_service::{MockVerificationService, TokenVerifier}}, try_or};

    use actix_web::{App, test::{self, TestRequest}, web::Data};
    use sqlx::{Executor, Pool, Sqlite};

    async fn setup_user(pool: &Data<Pool<Sqlite>>) -> anyhow::Result<bool>{
        match pool.execute("
            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token)
            VALUES ('test1','test@test.com','pwd','pfp',
            '69',
            '69');

            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token)
            VALUES ('test2','test1@test.com','pwd','pfp',
            '70',
            '70');
        ").await {
            Ok(a) => return Ok(true),
            Err(e) => {
                dbg!(e);
                return Ok(false);
            }
        }
    }

    async fn clean_db(pool: &Data<Pool<Sqlite>>){
        let _ = pool.execute("DELETE FROM friends;").await;
        let _ = pool.execute("DELETE FROM firend_requests;").await;
        let _ = pool.execute("DELETE FROM user;").await;

        // Reset AUTOINCREMENT counters (important for IDs = 1)
        let _ = pool.execute("DELETE FROM sqlite_sequence;").await;
    }
    async fn fresh_start(pool: &Data<Pool<Sqlite>>){
        clean_db(&pool).await;
        match setup_user(&pool).await{
            Ok(_)=>{},
            Err(e)=>{
                panic!("{e}");
            }
        }
    }
    #[actix_web::test]
    async fn test_send_friend_request(){
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> =
            Arc::new(MockVerificationService { should_return: true });
        
        let app = test::init_service(App::new()
        .app_data(pool.clone())
        .app_data(Data::from(verifier.clone()))
        .service(send_friend_request)).await;

        let req = TestRequest::post().uri("/send_friend_request")
        .insert_header(("Authorization", "69"))
        .set_json(&FriendRequest{
            user_id: 1,
            friend_id: 2,
        }).to_request();

        // send_friend_request should add the entry to the database
        let resp = test::call_service(&app, req).await;
        dbg!(resp.status());
        assert!(resp.status().is_success()); // make sure the service is run

        let row = sqlx::query("SELECT sender_id, receiver_id FROM friend_requests")
        .fetch_one(pool.as_ref())
        .await
        .expect("Friend request row not found");
        
        let reciever_id:i64 = row.try_get("receiver_id").expect("Failed to fetch receiver_id");
        let sender_id:i64 = row.try_get("sender_id").expect("Failed to fetch sender id");
        assert_eq!(reciever_id, 2); // make sure that the function has actually added to the db.
        assert_eq!(sender_id, 1);

        // send_freidn_request should return unauthorised if the token is invalid.




    }

    #[actix_web::test]
    async fn test_unauthorised_send_friend_request(){
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> =
            Arc::new(MockVerificationService { should_return: false });

        let app = test::init_service(App::new()
        .app_data(pool.clone())
        .app_data(Data::from(verifier.clone()))
        .service(send_friend_request)).await;

        let req = TestRequest::post().uri("/send_friend_request")
        .insert_header(("Authorization", "69"))
        .set_json(&FriendRequest{
            user_id: 69,
            friend_id: 70,
        }).to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status().as_u16(), StatusCode::UNAUTHORIZED.as_u16());
    }


}