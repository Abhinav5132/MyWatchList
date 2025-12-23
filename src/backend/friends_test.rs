#[cfg(test)]
mod tests{
    use std::sync::Arc;
    use reqwest::StatusCode;
    use sqlx::{Pool, Sqlite, sqlite, *};
    use crate::{backend::{friends::{FriendRequest, RequestId, accept_friend_request, decline_friend_request, send_friend_request}, setup_db, verification_service::{MockVerificationService, TokenVerifier}}, try_or};

    use actix_web::{App, test::{self, TestRequest}, web::Data};

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

    #[actix_web::test]
    async fn test_unauthorised_accept_friend_request(){
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> =
            Arc::new(MockVerificationService { should_return: false });

        let app = test::init_service(App::new()
        .app_data(pool.clone())
        .app_data(Data::from(verifier.clone()))
        .service(accept_friend_request)).await;

        let req = TestRequest::post().uri("/accept_friend_request")
        .insert_header(("Authorization", "69"))
        .set_json(&RequestId{
            request_id:1
        }).to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status().as_u16(), StatusCode::UNAUTHORIZED.as_u16());
    }

    #[actix_web::test]
    async fn test_accept_friend_request(){
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> =
            Arc::new(MockVerificationService { should_return: true });

        let app = test::init_service(App::new()
        .app_data(pool.clone())
        .app_data(Data::from(verifier.clone()))
        .service(accept_friend_request)).await;

        let id = sqlx::query("INSERT INTO friend_requests(sender_id, receiver_id) VALUES (?,?)")
        .bind(1).bind(2).execute(pool.as_ref()).await.expect("Failed to insert test friend_request into db").last_insert_rowid();
        
        let req = TestRequest::post().uri("/accept_friend_request")
        .insert_header(("Authorization", "69"))
        .set_json(&RequestId{
            request_id: id
        }).to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let row = sqlx::query("SELECT user_1, user_2 FROM friends")
        .fetch_one(pool.as_ref()).await.expect("Failed to fetch from the database.");
        
        let user1:i64 = row.try_get("user_1").expect("failed to decode user1");
        let user2:i64 = row.try_get("user_2").expect("failed to decode user2");

        if !((user1 == 1 && user2 ==2) || (user1 == 2 && user2 == 1)){
            panic!("User ID verification Failed")
        }
    }

    #[actix_web::test]
    async fn test_decline_friend_request(){
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService { should_return: true });

        let app = test::init_service(App::new()
        .app_data(pool.clone())
        .app_data(Data::from(verifier.clone()))
        .service(decline_friend_request)).await;

        let id = sqlx::query("INSERT INTO friend_requests(sender_id, receiver_id) VALUES (?,?)")
        .bind(1).bind(2).execute(pool.as_ref()).await.expect("Failed to insert test friend_request into db").last_insert_rowid();
        
        let req = TestRequest::post().uri("/decline_friend_request")
        .insert_header(("Authorization", "69"))
        .set_json(&RequestId{
            request_id: id
        }).to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let row = sqlx::query("SELECT user_1, user_2 FROM friends")
        .bind(1).bind(2)
        .fetch_one(pool.as_ref()).await;
        assert!(row.is_err());
        
        let row = sqlx::query("SELECT * FROM friend_requests WHERE id = ?").bind(id)
        .fetch_one(pool.as_ref()).await;
        assert!(row.is_err());

    }

    #[actix_web::test]
    async fn test_unauthorised_decline_friend_request(){
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> =
            Arc::new(MockVerificationService { should_return: false });

        let app = test::init_service(App::new()
        .app_data(pool.clone())
        .app_data(Data::from(verifier.clone()))
        .service(decline_friend_request)).await;

        let req = TestRequest::post().uri("/decline_friend_request")
        .insert_header(("Authorization", "69"))
        .set_json(&RequestId{
            request_id:1
        }).to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status().as_u16(), StatusCode::UNAUTHORIZED.as_u16());
    }
}