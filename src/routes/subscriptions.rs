use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// T in Form<T> must implement DeserializeOwned from serde, which will deserialize
// a url-encoded string into that struct
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_prefix = format!("[request_id {request_id}] - ");
    // create a span that will live for the duration of the request
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        // The '%' indicates to `tracing` that we want Display used to render the
        // values
        request_id = %request_id, // equivalent to request_id = %request_id
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    // After creating a span
    //
    // .enter() shouldn't be used in async code!
    let _request_span_guard = request_span.enter();

    // create a span specifically for tracing the database query
    let query_span = tracing::info_span!("Saving new subscriber details to database");

    let result = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .instrument(query_span)
    .await;

    match result {
        Ok(_) => {
            tracing::info!("{}Subscriber saved to database", request_prefix);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            // use Debug to log the error - we want more information in the logs
            // than Display would show
            tracing::error!("{}Failed to execute query: {:?}", request_prefix, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
