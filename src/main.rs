use bollard::{
    container::{AttachContainerOptions, AttachContainerResults, ListContainersOptions},
    Docker,
};
use tokio::{self, io::AsyncWriteExt};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let docker = Docker::connect_with_http_defaults().unwrap();
    let containers = docker
        .list_containers(Some(ListContainersOptions::<String> {
            ..Default::default()
        }))
        .await
        .unwrap();

    let (tx, mut rx1) = tokio::sync::broadcast::channel(8);

    for c in containers {
        let AttachContainerResults {
            mut output,
            mut input,
        } = docker
            .attach_container(
                &c.id.clone().unwrap(),
                Some(AttachContainerOptions::<String> {
                    stdin: Some(true),
                    stdout: Some(true),
                    stream: Some(true),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();

        // input.write("tellraw @a [{\"text\":\"[server1] \",\"color\":\"red\"},{\"text\":\"<player1> \",\"color\":\"blue\"},{\"text\":\"this is the message\",\"color\":\"white\"}]\n".as_bytes()).await.ok();

        let my_id = c.id.clone().unwrap();
        let my_other_id = my_id.clone();
        let my_tx = tx.clone();
        tokio::spawn(async move {
            while let Some(Ok(output)) = output.next().await {
                my_tx.send((my_id.clone(), output)).unwrap();
            }
        });

        let mut my_rx = tx.subscribe();
        tokio::spawn(async move {
            loop {
                let (server, msg) = my_rx.recv().await.unwrap();
                if server == my_other_id {
                    continue;
                }
                input
                    .write(format!("/say {}\n", msg.to_string().trim()).as_bytes())
                    .await
                    .unwrap();
            }
        });
    }

    loop {
        dbg!(rx1.recv().await.unwrap());
    }
}

/*
 * Docker container
 * -> Listening to Docker logs
 *   -> tx on broadcast
 * -> Listening to broadcast rx
 *   -> push to Docker attach
 *
 * Main thread
 * -> Get containers
 * -> Start threads per container
 *   -> Watch docker output logs
 *   -> Wait for broadcast
 */
