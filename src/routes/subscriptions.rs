use actix_web::{HttpResponse, web};
use chrono::Utc;
use sqlx::{PgPool};
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;
use crate::domain::NewSubscriber;
use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub name: String,
    pub email: String,
}


pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {

    let name = SubscriberName::parse(form.name)?;
    let email = SubscriberEmail::parse(form.email)?;

    Ok(NewSubscriber { email, name })
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, connection),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )

)]
pub async fn subscribe (form: web::Form<FormData>, connection: web::Data<PgPool> ) -> HttpResponse {


    

    let new_subscriber = match form.0.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber(&connection, &new_subscriber)
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
    skip(connection, new_subscriber)
)]
pub async fn insert_subscriber(
    connection: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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

pub fn is_valid_name(s: &str) -> bool {

    let is_empty_or_whitespace = s.trim().is_empty();

    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}