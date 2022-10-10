use std::{sync::atomic::{AtomicBool, Ordering::Relaxed}, future::pending};

use zbus::{dbus_interface, ConnectionBuilder, SignalContext};

fn main()->anyhow::Result<()> {
    let rt=tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    rt.block_on(async{
        let server=Server{suppressed:AtomicBool::new(false)};
        let _=ConnectionBuilder::session()?
            .name("com.github.robinmarchart.mprisutils")?
            .serve_at("/com/github/robinmarchart/mprisutils", server)?
            .build().await?;
        pending::<anyhow::Result<()>>().await
    })
}

struct Server{
    suppressed:AtomicBool
}

#[dbus_interface(name="com.github.robinmarchart.mprisutils")]
impl Server{

    #[dbus_interface(property)]
    fn suppressed(&self)->bool{
        self.suppressed.load(Relaxed)
    }

    #[dbus_interface(property)]
    fn set_suppressed(&self,new:bool){
        self.suppressed.store(new, Relaxed)
    }

    async fn toggle(
        &self,
        #[zbus(signal_context)]
        ctxt: SignalContext<'_>
    )->bool{
        self.suppressed.fetch_xor(true, Relaxed);
        self.suppressed_changed(&ctxt)
            .await
            .err()
            .into_iter()
                            .for_each(|e|eprintln!("{}",e));
        self.suppressed.load(Relaxed)

    }
}
