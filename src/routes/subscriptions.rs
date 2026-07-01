use actix_web::{HttpResponse, Responder, web};
use sqlx::{PgPool};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )

)]
pub async fn subscribe (form: web::Form<FormData>, connection: web::Data<PgPool> ) -> impl Responder {


    match insert_subscriber(&connection, &form)
    .await {

        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(_e) => {
            HttpResponse::InternalServerError().finish()
        }
    }
    
    
}


#[tracing::instrument(
    name = "Saving new subscriber details in the database.",
    skip(connection, form)
)]
pub async fn insert_subscriber(
    connection: &PgPool,
    form: &FormData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}