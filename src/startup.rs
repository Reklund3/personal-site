use crate::authentication::reject_anonymous_users;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::*;
use actix_files::Files;
use actix_session::config::PersistentSession;
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::middleware::from_fn;
use actix_web::web::Data;
use actix_web::{middleware, web, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        // Postgres pool
        let pg_pool: Pool<Postgres> = get_pg_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let email_client_timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url.clone(),
            sender_email,
            configuration.email_client.authorization_token.clone(),
            email_client_timeout,
        );

        // tls config
        // let tls_enabled = configuration.application.tls_enabled;
        let tls_config: Option<rustls::ServerConfig> = if configuration.application.tls_enabled {
            Some(load_rustls_config(
                configuration.application.cert_file_path.as_str(),
                configuration.application.key_file_path.as_str(),
            ))
        } else {
            None
        };

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, pg_pool, email_client, configuration, tls_config).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_pg_pool(database_configuration: &DatabaseSettings) -> PgPool {
    // Postgres pool
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(database_configuration.connect_options())
}

fn create_security_headers(tls_enabled: bool) -> middleware::DefaultHeaders {
    let mut headers = middleware::DefaultHeaders::new()
        .add(("X-Frame-Options", "DENY"))
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
        .add(("X-XSS-Protection", "1; mode=block"))
        .add((
            "Content-Security-Policy",
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; \
             font-src 'self'; \
             connect-src 'self'; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'",
        ))
        .add((
            "Permissions-Policy",
            "geolocation=(), microphone=(), camera=()",
        ));

    // Add HSTS header when TLS is enabled
    if tls_enabled {
        headers = headers.add((
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains; preload",
        ));
    }

    headers
}

async fn run(
    listener: TcpListener,
    pg_pool: PgPool,
    email_client: EmailClient,
    configuration: Settings,
    tls_config: Option<rustls::ServerConfig>,
) -> Result<Server, anyhow::Error> {
    let db_pool = Data::new(pg_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(configuration.application.base_url.clone());
    let resume_config = Data::new(ResumeConfig {
        file_path: configuration.application.resume_file_path,
    });
    let headshot_config = Data::new(HeadshotConfig {
        file_path: configuration.application.headshot_file_path,
    });
    let secret_key = Key::from(
        configuration
            .application
            .hmac_secret
            .expose_secret()
            .as_bytes(),
    );
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let redis_store = RedisSessionStore::new(configuration.redis_uri.expose_secret()).await?;
    let tls_enabled = tls_config.is_some();

    let server_builder = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(create_security_headers(tls_enabled))
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), secret_key.clone())
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(Duration::seconds(3600)),
                    )
                    .cookie_name("roberteklund".to_string())
                    .cookie_secure(tls_enabled)
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::Strict)
                    .cookie_path("/".to_string())
                    .build(),
            )
            .wrap(middleware::Compress::default())
            .wrap(TracingLogger::default())
            .service(web::resource("/").to(home))
            .service(
                web::scope("/admin")
                    .wrap(from_fn(reject_anonymous_users))
                    .route("/dashboard", web::get().to(admin_dashboard))
                    .route("/newsletters", web::get().to(publish_newsletter_form))
                    .route("/newsletters", web::post().to(publish_newsletter))
                    .route("/password", web::get().to(change_password_form))
                    .route("/password", web::post().to(change_password))
                    .route("/logout", web::post().to(log_out)),
            )
            .route("/contact", web::post().to(contact))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/resume", web::get().to(serve_resume))
            .route("/headshot", web::get().to(serve_headshot))
            .route("/robots.txt", web::get().to(serve_robots_txt))
            .route("/sitemap.xml", web::get().to(serve_sitemap_xml))
            .route("/ai.txt", web::get().to(serve_ai_txt))
            .service(
                Files::new("/", "./ui/dist/")
                    .index_file("index.html")
                    .use_etag(true)
                    .use_last_modified(true),
            )
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(resume_config.clone())
            .app_data(headshot_config.clone())
            .app_data(Data::new(HmacSecret(
                configuration.application.hmac_secret.clone(),
            )))
    });

    let server = match tls_config {
        Some(tls) => server_builder.listen_rustls_0_23(listener, tls)?.run(),
        None => {
            tracing_log::log::warn!("The server has been started without tls");
            server_builder.listen(listener)?.run()
        }
    };
    Ok(server)
}

fn load_rustls_config(cert_file_path: &str, key_file_path: &str) -> rustls::ServerConfig {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    // init server config builder with safe defaults
    let config = rustls::ServerConfig::builder().with_no_client_auth();

    // convert files to key/cert objects
    let cert_chain =
        CertificateDer::from_pem_file(cert_file_path).expect("Failed to load certificate chain.");
    let private_key =
        PrivateKeyDer::from_pem_file(key_file_path).expect("Failed to load private key.");

    config
        .with_single_cert(vec![cert_chain], private_key)
        .unwrap()
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
