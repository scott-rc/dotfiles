use kube::config::{KubeConfigOptions, Kubeconfig};

pub async fn build_client(context: Option<&str>) -> kube::Client {
    let kubeconfig = Kubeconfig::read().unwrap_or_else(|e| {
        eprintln!("boom: {e}");
        std::process::exit(1);
    });

    let options = KubeConfigOptions {
        context: context.map(String::from),
        ..Default::default()
    };

    let config = kube::Config::from_custom_kubeconfig(kubeconfig, &options)
        .await
        .unwrap_or_else(|e| {
            eprintln!("boom: {e}");
            std::process::exit(1);
        });

    kube::Client::try_from(config).unwrap_or_else(|e| {
        eprintln!("boom: {e}");
        std::process::exit(1);
    })
}
