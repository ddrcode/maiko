use maiko::{Actor, Context, Topic};

#[derive(Clone, PartialEq, Eq, Hash)]
enum Exchange {
    Alpha,
    Beta,
}

#[derive(Clone)]
enum Side {
    Buy,
    Sell,
}

#[derive(Clone)]
struct Tick {
    ex: Exchange,
    price: f64,
    size: usize,
    side: Side,
}

#[derive(Clone, Default)]
struct TopOfBook {
    bid: f64,
    bid_size: f64,
    ask: f64,
    ask_size: f64,
}

#[derive(maiko::Event, Clone)]
enum MarketEvent {
    AlphaTick {
        price: f64,
        quantity: u32,
    },
    BetaTick {
        price: String,
        side: char,
        count: u64,
    },
    MarketTick(Tick),
    Order(Tick),
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum MarketTopic {
    RawData,
    NormalizedData,
    Order(Exchange),
}

impl Topic<MarketEvent> for MarketTopic {
    fn from_event(event: &MarketEvent) -> Self {
        use MarketEvent::*;
        match event {
            AlphaTick { .. } => MarketTopic::RawData,
            BetaTick { .. } => MarketTopic::RawData,
            MarketTick(_) => MarketTopic::NormalizedData,
            Order(tick) => MarketTopic::Order(tick.ex.clone()),
        }
    }
}

struct Ticker;
impl Actor for Ticker {
    type Event = MarketEvent;
}

struct Normalizer {
    ctx: Context<MarketEvent>,
}
impl Actor for Normalizer {
    type Event = MarketEvent;
    async fn handle_event(&mut self, envelope: &maiko::Envelope<Self::Event>) -> maiko::Result<()> {
        let tick = match &envelope.event() {
            MarketEvent::AlphaTick { price, quantity } => Tick {
                ex: Exchange::Alpha,
                price: *price,
                size: *quantity as usize,
                side: Side::Buy,
            },
            MarketEvent::BetaTick { price, side, count } => {
                let side_enum = match side {
                    'B' => Side::Buy,
                    'S' => Side::Sell,
                    _ => Side::Buy,
                };
                let price_f64 = price.parse::<f64>().unwrap_or(0.0);
                Tick {
                    ex: Exchange::Beta,
                    price: price_f64,
                    size: *count as usize,
                    side: side_enum,
                }
            }
            _ => return Ok(()),
        };
        self.ctx.send(MarketEvent::MarketTick(tick)).await?;
        Ok(())
    }
}

struct Trader {
    ctx: Context<MarketEvent>,
    alpha_top: Option<TopOfBook>,
    beta_top: Option<TopOfBook>,
}
impl Trader {
    async fn process_tick(&mut self, tick: &Tick) {
        let top = match tick.ex {
            Exchange::Alpha => self.alpha_top.get_or_insert_default(),
            Exchange::Beta => self.beta_top.get_or_insert_default(),
        };
        match tick.side {
            Side::Buy => {
                top.bid = tick.price;
                top.bid_size += tick.size as f64;
            }
            Side::Sell => {
                top.ask = tick.price;
                top.ask_size += tick.size as f64;
            }
        }
    }

    fn try_arbitrage(&mut self) -> Some(Tick) {
        if let (Some(alpha), Some(beta)) = (&self.alpha_top, &self.beta_top) {
            if alpha.ask < beta.bid {
                Some(Tick {
                    ex: Exchange::Alpha,
                    price: alpha.ask,
                    size: 100,
                    side: Side::Sell,
                })
            } else if beta.ask < alpha.bid {
                Some(Tick {
                    ex: Exchange::Beta,
                    price: beta.ask,
                    size: 100,
                    side: Side::Sell,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Actor for Trader {
    type Event = MarketEvent;
    async fn handle_event(&mut self, envelope: &maiko::Envelope<Self::Event>) -> maiko::Result<()> {
        if let MarketEvent::MarketTick(tick) = &envelope.event() {
            self.process_tick(tick).await;
            if let Some(order_tick) = self.try_arbitrage() {
                self.ctx
                    .send(MarketEvent::Order(Tick {
                        ex: match order_tick.ex {
                            Exchange::Alpha => Exchange::Beta,
                            Exchange::Beta => Exchange::Alpha,
                        },
                        price: order_tick.price,
                        size: order_tick.size,
                        side: match order_tick.side {},
                    }))
                    .await?;
                self.ctx.send(MarketEvent::Order(order_tick)).await?;
            }
        }
        Ok(())
    }
}

struct Database;
impl Actor for Database {
    type Event = MarketEvent;
}

struct Telemetry;
impl Actor for Telemetry {
    type Event = MarketEvent;
}

#[tokio::main]
async fn main() {
    use Exchange::*;
    use MarketTopic::*;

    let mut sup = maiko::Supervisor::<MarketEvent, MarketTopic>::default();

    sup.add_actor("AlphaTicker", |_ctx| Ticker, &[Order(Alpha)]);
    sup.add_actor("BetaTicker", |_ctx| Ticker, &[Order(Beta)]);
    sup.add_actor("Normalizer", |ctx| Normalizer { ctx }, &[RawData]);
    sup.add_actor("Trader", |_ctx| Trader, &[NormalizedData]);
    sup.add_actor("Database", |_ctx| Trader, &[NormalizedData]);
    sup.add_actor(
        "Telemetry",
        |_ctx| Telemetry,
        &[RawData, NormalizedData, Order(Alpha), Order(Beta)],
    );
}
