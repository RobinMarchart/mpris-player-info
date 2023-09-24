use anyhow::Context;
use mpris_dbus::proxies::HideStateProxy;
use zbus::Connection;

pub fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build().context("building tokio runtime")?
        .block_on(async {
            let conn = Connection::session().await.context("connecting to session dbus")?;
            let proxy = HideStateProxy::new(&conn).await.context("connecting to hide state server")?;
            proxy.toggle().await.context("toggling hide state")?;
            Ok(())
        })
}
