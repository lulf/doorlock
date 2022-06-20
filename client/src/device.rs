use btleplug::api::{BDAddr, Central, Characteristic, Peripheral as _, WriteType};
use btleplug::platform::{Adapter, Peripheral};
use tokio::time::{sleep, Duration};

pub struct LockDevice {
    adapter: Adapter,
    address: BDAddr,
    board: Option<Peripheral>,
}

const LOCK_SERVICE_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002000b0cd11ec871fd45ddf138840);

const LOCK_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002001b0cd11ec871fd45ddf138840);
const SPEED_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002002b0cd11ec871fd45ddf138840);
const STEP_CHAR_UUID: uuid::Uuid = uuid::Uuid::from_u128(0x00002003b0cd11ec871fd45ddf138840);

impl LockDevice {
    pub fn new(device: &str, adapter: Adapter) -> Self {
        Self {
            address: BDAddr::from_str_delim(device).unwrap(),
            adapter,
            board: None,
        }
    }

    pub async fn lock(&mut self) -> anyhow::Result<()> {
        self.write_char(LOCK_SERVICE_UUID, LOCK_CHAR_UUID, &[1]).await
    }

    pub async fn unlock(&mut self) -> anyhow::Result<()> {
        self.write_char(LOCK_SERVICE_UUID, LOCK_CHAR_UUID, &[0]).await
    }

    pub async fn set_speed(&mut self, speed: u32) -> anyhow::Result<()> {
        let data = speed.to_le_bytes();
        self.write_char(LOCK_SERVICE_UUID, SPEED_CHAR_UUID, &data[..]).await
    }

    pub async fn set_step(&mut self, step: u16) -> anyhow::Result<()> {
        let data = step.to_le_bytes();
        self.write_char(LOCK_SERVICE_UUID, STEP_CHAR_UUID, &data[..]).await
    }

    #[allow(dead_code)]
    async fn read_char(&mut self, service: uuid::Uuid, c: uuid::Uuid) -> anyhow::Result<Vec<u8>> {
        let (device, c) = self.find_char(service, c).await?;
        if let Some(c) = c {
            let value = device.read(&c).await?;
            Ok(value)
        } else {
            Err(anyhow::anyhow!("unable to locate characteristic"))
        }
    }

    async fn write_char(&mut self, service: uuid::Uuid, c: uuid::Uuid, value: &[u8]) -> anyhow::Result<()> {
        let (device, c) = self.find_char(service, c).await?;
        if let Some(c) = c {
            device.write(&c, value, WriteType::WithResponse).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("unable to locate characteristic"))
        }
    }

    async fn find_char(
        &mut self,
        service: uuid::Uuid,
        characteristic: uuid::Uuid,
    ) -> anyhow::Result<(&mut Peripheral, Option<Characteristic>)> {
        let device = self.connect().await?;
        for s in device.services() {
            if s.uuid == service {
                for c in s.characteristics {
                    if c.uuid == characteristic {
                        return Ok((device, Some(c)));
                    }
                }
            }
        }
        Ok((device, None))
    }

    async fn connect(&mut self) -> anyhow::Result<&mut Peripheral> {
        if self.board.is_none() {
            loop {
                for device in self.adapter.peripherals().await? {
                    if let Some(p) = device.properties().await? {
                        if p.address == self.address {
                            // Make sure we get a fresh start
                            let _ = device.disconnect().await;
                            sleep(Duration::from_secs(2)).await;
                            match device.is_connected().await {
                                Ok(false) => {
                                    log::info!("Connecting...");
                                    loop {
                                        match device.connect().await {
                                            Ok(()) => break,
                                            Err(err) => {
                                                log::error!("Connect error: {}", &err);
                                            }
                                        }
                                    }
                                    log::info!("Connected!");
                                    device.discover_services().await?;
                                    self.board.replace(device);
                                    return Ok(self.board.as_mut().unwrap());
                                }
                                Ok(true) => {
                                    log::info!("Connected!");
                                    self.board.replace(device);
                                    return Ok(self.board.as_mut().unwrap());
                                }
                                Err(e) => {
                                    log::info!("Error checking connection, retrying: {:?}", e);
                                }
                            }
                        }
                    }
                }
                sleep(Duration::from_secs(2)).await;
            }
        }
        Ok(self.board.as_mut().unwrap())
    }
}
