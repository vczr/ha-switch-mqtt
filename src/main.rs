mod ha;
use ha::InitEntity;
use rumqttc::{AsyncClient, Client, ClientError, ConnectionError, Event, EventLoop, Incoming, MqttOptions, QoS};
use std::string::ToString;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    //初始化系统状态为OFF
    let state: Arc<Mutex<&str>> = Arc::new(Mutex::new("OFF"));
    //初始化配置文件，设定当前设备的系统名字
    let mut mqtt_config = MqttConfig::new("虚拟开关2".to_string(),"mqtt_switch_01".to_string());
    //链接mqtt broker
    let (mut client, mut conn) = MqttClient::init(mqtt_config.clone());
    //注册本实体
    init_entity(&mut mqtt_config, &client).await;
    //把config变成共享的
    let arc_mqtt_config_main = Arc::new(mqtt_config.clone());

    //多线程处理client
    let arc_client = Arc::new(client);
    //订阅命令topic
    println!("开始监听{}", arc_mqtt_config_main.command_topic.clone());
    Arc::clone(&arc_client).subscribe(arc_mqtt_config_main.command_topic.clone(), QoS::AtMostOnce).await.unwrap();
    //处理共享变量
    let client_t1 = Arc::clone(&arc_client);
    let mut state_t1 = Arc::clone(&state);
    let arc_mqtt_config_listener_thread = Arc::clone(&arc_mqtt_config_main);

    //新建一个线程用来监听
    tokio::spawn(async move {
        loop {
            let event = conn.poll().await;
            match event {
                Ok(ev) => {
                    // println!("{:?}", ev);
                    match ev {
                        Event::Incoming(incoming) => {
                            match incoming {
                                Incoming::Publish(publish) => {
                                    let payload = std::str::from_utf8(&publish.payload).unwrap();
                                    println!("收到命令: {}", payload);
                                    if (payload != "ON" && payload != "OFF") {
                                        println!("非法的命令");
                                        continue;
                                    }
                                    //TODO 执行
                                    //修改系统状态
                                    *state_t1.lock().unwrap() = {
                                        if payload == "ON" {
                                            "ON"
                                        } else {
                                            "OFF"
                                        }
                                    };
                                    client_t1.publish(arc_mqtt_config_listener_thread.state_topic.to_string(), QoS::AtMostOnce, false, payload).await.unwrap();
                                }
                                _ => {}
                            }
                        }
                        Event::Outgoing(_) => {}
                    }
                }
                Err(err) => {
                    println!("监听失败 {}", err);
                    continue;
                }
            }


        }
    });

    let client_t2 = Arc::clone(&arc_client);
    let state_t2 = Arc::clone(&state);
    let state_topic_t2 = arc_mqtt_config_main.clone();

    //主线程用来上报当前状态和心跳
    loop {
        match client_t2.publish(mqtt_config.availability_topic.clone(), QoS::AtMostOnce, false, "online").await {
            Ok(_) => {
                println!("发送心跳成功");
            }
            Err(e) => {
                println!("心跳发送失败 {}",e)
            }
        };
        println!("上报当前状态:{}", *state_t2.lock().unwrap());
        client_t2.publish(state_topic_t2.state_topic.to_string(), QoS::AtMostOnce, false, *state_t2.lock().unwrap()).await.unwrap();
        sleep(Duration::from_secs(5)).await;
    }
}

async fn init_entity (config: &mut MqttConfig, client : &AsyncClient) {
    let payload = InitEntity::new(
        {
            if config.name == String::new() {
                config.driver_name.clone()
            }
            else {
                config.name.clone()
            }
        },
        config.driver_name.clone()
    );
    println!("注册实体 {}", serde_json::to_string(&payload).unwrap());
    client.publish(payload.config_topic.clone(), QoS::AtMostOnce, false, serde_json::to_string(&payload).unwrap()).await.expect("TODO: panic message");
    config.state_topic = payload.state_topic.clone();
    config.command_topic = payload.command_topic.clone();
    config.availability_topic = payload.availability_topic.clone();
    config.config_topic = payload.config_topic.clone();
}

#[derive(Clone)]
struct MqttConfig {
    name:String,
    driver_name: String,
    config_topic: String,
    state_topic: String,
    command_topic: String,
    availability_topic: String,
    broker_url: String,
    broker_port: u16,
    broker_username: String,
    broker_password: String,
}



impl MqttConfig {
    fn new(name :String ,driver_name: String) -> MqttConfig {
        let driver = driver_name.clone();
        MqttConfig {
            name,
            driver_name,
            config_topic: String::new(),
            state_topic: String::new(),
            command_topic: String::new(),
            availability_topic: String::new(),
            broker_url: "127.0.0.1".to_string(),
            broker_port: 11883,
            broker_username: "admin".to_string(),
            broker_password: "public".to_string(),
        }
    }
}

#[derive(Clone)]
struct MqttClient {
    config: MqttConfig,
    client: Client,
}

impl MqttClient{
    fn init(config : MqttConfig) -> (AsyncClient, EventLoop) {
        let mut options = MqttOptions::new(config.driver_name, config.broker_url, config.broker_port);
        options.set_credentials(config.broker_username,config.broker_password);
        AsyncClient::new(options, 10)
    }

}

