use askama::Template;
use rust_embed::RustEmbed;
use salvo::conn::tcp::TcpAcceptor;
use salvo::prelude::{Logger, Request, Response, Text};
use salvo::serve_static::static_embed;
use salvo::{handler, Router, Server};
use std::net::Ipv4Addr;
use std::str::FromStr;
use tracing::Level;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::EnvFilter;

#[handler]
async fn style(res: &mut Response) {
    res.render(Text::Css(include_str!(concat!(
        env!("OUT_DIR"),
        "/style.css"
    ))));
}

#[handler]
async fn htmx(res: &mut Response) {
    res.render(Text::Js(include_str!(
        "../node_modules/htmx.org/dist/htmx.min.js"
    )));
}

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/static"]
struct StaticFiles;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[handler]
async fn hello(req: &mut Request, res: &mut Response) {
    let hello_tmpl = HelloTemplate {
        name: req.query("name").unwrap_or("World"),
    };
    res.render(Text::Html(hello_tmpl.render().unwrap()));
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    setup_logging()?;

    let router = Router::new()
        .hoop(Logger::default())
        .push(Router::with_path("style.css").get(style))
        .push(Router::with_path("htmx.min.js").get(htmx))
        .push(Router::with_path("static/<**path>").get(static_embed::<StaticFiles>()))
        .get(hello);

    let port = u16::from_str(option_env!("PORT").unwrap_or("8080"))?;
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
    Server::new(TcpAcceptor::try_from(listener)?)
        .serve(router)
        .await;

    Ok(())
}

fn setup_logging() -> eyre::Result<()> {
    if cfg!(debug_assertions) {
        let filter = EnvFilter::builder()
            .with_default_directive(Level::DEBUG.into())
            .from_env_lossy()
            .add_directive(Directive::from_str("hyper=info")?);

        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .pretty()
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .without_time()
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        let filter = EnvFilter::builder()
            .with_default_directive(Level::INFO.into())
            .from_env_lossy();

        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .compact()
            .with_thread_names(true)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
    }

    Ok(())
}
