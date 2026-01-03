#[cfg(test)]
mod tests {
    use crate::{
        backend::{
            friends::{
                AllFriendRequests, AllFriends, FriendId, FriendRequest, FriendRequestDirection,
                RequestId, accept_friend_request, decline_friend_request, get_all_friends,
                get_all_friends_requests, remove_friend, send_friend_request,
            },
            setup_db,
            verification_service::{MockVerificationService, TokenVerifier},
        },
    };
    use reqwest::StatusCode;
    use sqlx::{Pool, Sqlite, *};
    use std::sync::Arc;

    use actix_web::{
        App,
        test::{self, TestRequest},
        web::Data,
    };

    async fn setup_user(pool: &Data<Pool<Sqlite>>) -> anyhow::Result<bool> {
        match pool.execute("
            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token, chosen_update_schedule)
            VALUES ('test1','test@test.com','pwd','pfp',
            '69',
            '69', 'NONE');

            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token, chosen_update_schedule)
            VALUES ('test2','test1@test.com','pwd','pfp',
            '70',
            '70', 'NONE');
        ").await {
            Ok(_) => Ok(true),
            Err(e) => {
                dbg!(e);
                Ok(false)
            }
        }
    }

    async fn clean_db(pool: &Data<Pool<Sqlite>>) {
        let _ = pool.execute("DELETE FROM friends;").await;
        let _ = pool.execute("DELETE FROM firend_requests;").await;
        let _ = pool.execute("DELETE FROM user;").await;

        // Reset AUTOINCREMENT counters (important for IDs = 1)
        let _ = pool.execute("DELETE FROM sqlite_sequence;").await;
    }
    async fn fresh_start(pool: &Data<Pool<Sqlite>>) {
        clean_db(pool).await;
        match setup_user(pool).await {
            Ok(_) => {}
            Err(e) => {
                panic!("{e}");
            }
        }
    }
    #[actix_web::test]
    async fn test_send_friend_request() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: true,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(send_friend_request),
        )
        .await;

        let req = TestRequest::post()
            .uri("/send_friend_request")
            .insert_header(("Authorization", "69"))
            .set_json(&FriendRequest {
                user_id: 1,
                friend_id: 2,
            })
            .to_request();

        // send_friend_request should add the entry to the database
        let resp = test::call_service(&app, req).await;
        dbg!(resp.status());
        assert!(resp.status().is_success()); // make sure the service is run

        let row = sqlx::query("SELECT sender_id, receiver_id FROM friend_requests")
            .fetch_one(pool.as_ref())
            .await
            .expect("Friend request row not found");

        let reciever_id: i64 = row
            .try_get("receiver_id")
            .expect("Failed to fetch receiver_id");
        let sender_id: i64 = row.try_get("sender_id").expect("Failed to fetch sender id");
        assert_eq!(reciever_id, 2); // make sure that the function has actually added to the db.
        assert_eq!(sender_id, 1);
    }

    #[actix_web::test]
    async fn test_unauthorised_send_friend_request() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: false,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(send_friend_request),
        )
        .await;

        let req = TestRequest::post()
            .uri("/send_friend_request")
            .insert_header(("Authorization", "69"))
            .set_json(&FriendRequest {
                user_id: 69,
                friend_id: 70,
            })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status().as_u16(), StatusCode::UNAUTHORIZED.as_u16());
    }

    #[actix_web::test]
    async fn test_unauthorised_accept_friend_request() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: false,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(accept_friend_request),
        )
        .await;

        let req = TestRequest::post()
            .uri("/accept_friend_request")
            .insert_header(("Authorization", "69"))
            .set_json(&RequestId { request_id: 1 })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status().as_u16(), StatusCode::UNAUTHORIZED.as_u16());
    }

    #[actix_web::test]
    async fn test_accept_friend_request() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: true,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(accept_friend_request),
        )
        .await;

        let id = sqlx::query("INSERT INTO friend_requests(sender_id, receiver_id) VALUES (?,?)")
            .bind(1)
            .bind(2)
            .execute(pool.as_ref())
            .await
            .expect("Failed to insert test friend_request into db")
            .last_insert_rowid();

        let req = TestRequest::post()
            .uri("/accept_friend_request")
            .insert_header(("Authorization", "69"))
            .set_json(&RequestId { request_id: id })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let row = sqlx::query("SELECT user_1, user_2 FROM friends")
            .fetch_one(pool.as_ref())
            .await
            .expect("Failed to fetch from the database.");

        let user1: i64 = row.try_get("user_1").expect("failed to decode user1");
        let user2: i64 = row.try_get("user_2").expect("failed to decode user2");

        if !((user1 == 1 && user2 == 2) || (user1 == 2 && user2 == 1)) {
            panic!("User ID verification Failed")
        }
    }

    #[actix_web::test]
    async fn test_decline_friend_request() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: true,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(decline_friend_request),
        )
        .await;

        let id = sqlx::query("INSERT INTO friend_requests(sender_id, receiver_id) VALUES (?,?)")
            .bind(1)
            .bind(2)
            .execute(pool.as_ref())
            .await
            .expect("Failed to insert test friend_request into db")
            .last_insert_rowid();

        let req = TestRequest::post()
            .uri("/decline_friend_request")
            .insert_header(("Authorization", "69"))
            .set_json(&RequestId { request_id: id })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let row = sqlx::query("SELECT user_1, user_2 FROM friends")
            .bind(1)
            .bind(2)
            .fetch_one(pool.as_ref())
            .await;
        assert!(row.is_err());

        let row = sqlx::query("SELECT * FROM friend_requests WHERE id = ?")
            .bind(id)
            .fetch_one(pool.as_ref())
            .await;
        assert!(row.is_err());
    }

    #[actix_web::test]
    async fn test_unauthorised_decline_friend_request() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: false,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(decline_friend_request),
        )
        .await;

        let req = TestRequest::post()
            .uri("/decline_friend_request")
            .insert_header(("Authorization", "69"))
            .set_json(&RequestId { request_id: 1 })
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status().as_u16(), StatusCode::UNAUTHORIZED.as_u16());
    }

    #[actix_web::test]
    async fn test_get_all_friends() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: true,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(get_all_friends),
        )
        .await;

        pool.execute("
            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token, chosen_update_schedule)
            VALUES ('test3','test3@test.com','pwd','pfp',
            '71',
            '71','NONE');

            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token, chosen_update_schedule)
            VALUES ('test4','test4@test.com','pwd','pfp',
            '72',
            '72','NONE');
        ").await.expect("Failed to insert other test users");

        let _ = sqlx::query(
            "
            INSERT INTO friends(user_1, user_2) VALUES (1,2);
            INSERT INTO friends(user_1, user_2) VALUES (1,3);
            INSERT INTO friends(user_1, user_2) VALUES (1,4);
            INSERT INTO friends(user_1, user_2) VALUES (2,4);
            INSERT INTO friends(user_1, user_2) VALUES (2,3);
        ",
        )
        .execute(pool.as_ref())
        .await
        .expect("Failed to insert test friend into db");

        let req = TestRequest::get()
            .uri("/get_all_friends")
            .insert_header(("Authorization", "69"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        dbg!(resp.status());
        assert!(resp.status().is_success());

        let body: AllFriends = test::read_body_json(resp).await;

        assert_eq!(body.friends.len(), 3); // kinda round about but it also tests for other firendships not being returned. 
        // since the length is 3 and we check that all three friends are returned are the right ones.

        assert_eq!(body.friends[0].friend_id, 2);
        assert_eq!(body.friends[0].user_name, "test2");

        assert_eq!(body.friends[1].friend_id, 3);
        assert_eq!(body.friends[1].user_name, "test3");

        assert_eq!(body.friends[2].friend_id, 4);
        assert_eq!(body.friends[2].user_name, "test4");
    }

    #[actix_web::test]
    async fn test_get_all_friend_requests() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: true,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(get_all_friends_requests),
        )
        .await;

        pool.execute("
            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token, chosen_update_schedule)
            VALUES ('test3','test3@test.com','pwd','pfp',
            '71',
            '71', 'NONE');

            Insert INTO user(user_name, user_email, user_password, user_pfp, user_access_token, user_refresh_token, chosen_update_schedule)
            VALUES ('test4','test4@test.com','pwd','pfp',
            '72',
            '72', 'NONE');
        ").await.expect("Failed to insert other test users");

        let _ = sqlx::query(
            "
            INSERT INTO friend_requests(sender_id, receiver_id) VALUES (1,2);
            INSERT INTO friend_requests(sender_id, receiver_id) VALUES (1,3);
            INSERT INTO friend_requests(sender_id, receiver_id) VALUES (4,1);
            INSERT INTO friend_requests(sender_id, receiver_id) VALUES (2,4);
            INSERT INTO friend_requests(sender_id, receiver_id) VALUES (3,2);

        ",
        ) //(4,1) is an incoming request and we much check for this
        .execute(pool.as_ref())
        .await
        .expect("Failed to insert test friend into db");

        let req = TestRequest::get()
            .uri("/get_all_friends_requests")
            .insert_header(("Authorization", "69"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        dbg!(resp.status());
        assert!(resp.status().is_success());

        let body: AllFriendRequests = test::read_body_json(resp).await;

        assert_eq!(body.friend_requests.len(), 3);

        let r2 = body
            .friend_requests
            .iter()
            .find(|r| r.friend_id == 2)
            .unwrap();
        assert_eq!(r2.user_name, "test2");
        assert_eq!(r2.direction, FriendRequestDirection::SENDING);

        let r3 = body
            .friend_requests
            .iter()
            .find(|r| r.friend_id == 3)
            .unwrap();
        assert_eq!(r3.user_name, "test3");
        assert_eq!(r3.direction, FriendRequestDirection::SENDING);

        let r4 = body
            .friend_requests
            .iter()
            .find(|r| r.friend_id == 4)
            .unwrap();
        assert_eq!(r4.user_name, "test4");
        assert_eq!(r4.direction, FriendRequestDirection::INCOMING);
    }

    #[actix_web::test]
    async fn test_remove_friend() {
        let pool = setup_db().await;
        fresh_start(&pool).await;

        let verifier: Arc<dyn TokenVerifier> = Arc::new(MockVerificationService {
            should_return: true,
        });

        let app = test::init_service(
            App::new()
                .app_data(pool.clone())
                .app_data(Data::from(verifier.clone()))
                .service(remove_friend),
        )
        .await;

        let id = sqlx::query(
            "
            INSERT INTO friends(user_1, user_2) VALUES (1,2);
        ",
        )
        .execute(pool.as_ref())
        .await
        .expect("Failed to insert friendship")
        .last_insert_rowid();

        let req = TestRequest::post()
            .uri("/remove_friend")
            .insert_header(("Authorization", "69"))
            .set_json(&FriendId { friendship_id: id })
            .to_request();

        let res = test::call_service(&app, req).await;

        assert!(res.status().is_success());

        let a = sqlx::query("SELECT user_1, user_2 FROM friends")
            .fetch_one(pool.as_ref())
            .await;
        assert!(a.is_err());
    }
}
