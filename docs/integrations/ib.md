# Interactive Brokers

NautilusTrader offers an adapter for integrating with the Interactive Brokers Gateway via 
[ibapi](https://github.com/nautechsystems/ibapi).

**Note**: If you are planning on using the built-in docker TWS Gateway when using the Interactive Brokers adapter,
you must ensure the `docker` package is installed. Run `poetry install --extras "ib docker"` 
or `poetry install --all-extras` inside your environment to ensure the necessary packages are installed.

## Quickstart
To run any interactive brokers strategies create account at [Interactive Brokers](https://www.interactivebrokers.com/en/home.php).
If you want to enable Paper trading, go to `Settings -> Account Configuration -> Paper Trading Account`.
There you need to request a paper trading account and wait for approval for few days. After the successful approval
you will get paper trading account number `DU**`. Try connecting to Portal with this paper trade account, and password.
If you can login to Portal, you can use this account for paper trading. In the main account, you also need to check `Share real-time market data subscriptions with paper trading account?`
to `Yes` which will share data subscriptions between main and paper trading account.
For the connectivity with NautilusTrader, you have two options:
1. **IB Gateway** (more of production use)
2. **TWS Workstation** (when you need to visualize at the same time what's happening in account) - 
this is more true when in paper account because you cannot have multi-login

### TWS Workstation
- Install [TWS gateway](https://www.interactivebrokers.com/en/trading/tws-updateable-latest.php) for your OS
- Run TWS gateway and login with your account (paper or real)
- Go to menu `File` and `Global Configuration` and go to `API` section. There you need to check option `Enable ActiveX and Socket Clients`
- You can also check `Read-Only API` if you want to use only data feed and not trading
- After that you can run NautilusTrader with IB integration which will connect to TWS gateway with API

### IB Gateway
TODO

### Run example strategy
Insert `username`, `password` and `account_id` into `interactive_brokers_example.py` and run it.


```python
username="***"
password="***"
account_id="DU***"

# add the credentials to the config
gateway = InteractiveBrokersGatewayConfig(
    start=False,
    username=username # <-- add your username here,
    password=password # <-- add your password here,
    trading_mode="paper",
    read_only_api=True,
)

# Change the exec client
    exec_clients={
        "IB": InteractiveBrokersExecClientConfig(
            ibg_host="127.0.0.1",
            ibg_port=7497,
            ibg_client_id=1,
            account_id=account_id,  # <-- add your account id here,
            gateway=gateway,
            instrument_provider=instrument_provider,
            routing=RoutingConfig(
                default=True,
            ),
        ),
    },
```



## Overview

The following integration classes are available:
- `InteractiveBrokersInstrumentProvider` which allows querying Interactive Brokers for instruments.
- `InteractiveBrokersDataClient` which connects to the `Gateway` and streams market data.
- `InteractiveBrokersExecutionClient` which allows the retrieval of account information and execution of orders.

## Instruments
Interactive Brokers allows searching for instruments via the `qualifyContracts` API, which, if given enough information
can usually resolve a filter into an actual contract(s). A node can request instruments to be loaded by passing 
configuration to the `InstrumentProviderConfig` when initialising a `TradingNodeConfig` (note that while `filters`
is a dict, it must be converted to a tuple when passed to `InstrumentProviderConfig`).

At a minimum, you must specify the `secType` (security type) and `symbol` (equities etc) or `pair` (FX). See examples 
queries below for common use cases 

Example config: 

```python
config_node = TradingNodeConfig(
    data_clients={
        "IB": InteractiveBrokersDataClientConfig(
            instrument_provider=InteractiveBrokersInstrumentProviderConfig(
                load_ids={"EUR/USD.IDEALPRO", "AAPL.NASDAQ"},
                load_contracts={IBContract(secType="CONTFUT", exchange="CME", symbol="MES")},
            )
    ),
    ...
)
```

### Examples queries
- Stock: `IBContract(secType='STK', exchange='SMART', symbol='AMD', currency='USD')`
- Stock: `IBContract(secType='STK', exchange='SMART', primaryExchange='NASDAQ', symbol='INTC')`
- Forex: `InstrumentId('EUR/USD.IDEALPRO')`, `InstrumentId('USD/JPY.IDEALPRO')`
- CFD: `IBContract(secType='CFD', symbol='IBUS30')`
- Future: `InstrumentId("YMH24.CBOT")`, `InstrumentId("CLZ27.NYMEX")`, `InstrumentId("ESZ27.CME")`, `InstrumentId('ES.CME')`, `IBContract(secType='CONTFUT', exchange='CME', symbol='ES', build_futures_chain=True)`
- Option: `InstrumentId('SPY251219C00395000.SMART')`, `IBContract(secType='STK', exchange='SMART', primaryExchange='ARCA', symbol='SPY', lastTradeDateOrContractMonth='20251219', build_options_chain=True)`
- Bond: `IBContract(secType='BOND', secIdType='ISIN', secId='US03076KAA60')`
- Crypto: `InstrumentId('BTC/USD.PAXOS')`


## Configuration
The most common use case is to configure a live `TradingNode` to include Interactive Brokers
data and execution clients. To achieve this, add an `IB` section to your client
configuration(s) and set the environment variables to your TWS (Traders Workstation) credentials:

```python
import os

config = TradingNodeConfig(
    data_clients={
        "IB": InteractiveBrokersDataClientConfig(
            username=os.getenv("TWS_USERNAME"),
            password=os.getenv("TWS_PASSWORD"),
            ...  # Omitted
    },
    exec_clients = {
        "IB": InteractiveBrokersExecutionClientConfig(
            username=os.getenv("TWS_USERNAME"),
            password=os.getenv("TWS_PASSWORD"),
            ...  # Omitted
    },
    ...  # Omitted
)
```

Then, create a `TradingNode` and add the client factories:

```python
# Instantiate the live trading node with a configuration
node = TradingNode(config=config)

# Register the client factories with the node
node.add_data_client_factory("IB", InteractiveBrokersLiveDataClientFactory)
node.add_exec_client_factory("IB", InteractiveBrokersLiveExecClientFactory)

# Finally build the node
node.build()
```

### API credentials
There are two options for supplying your credentials to the Interactive Brokers clients.
Either pass the corresponding `username` and `password` values to the config dictionaries, or
set the following environment variables: 
- `TWS_USERNAME`
- `TWS_PASSWORD`

When starting the trading node, you'll receive immediate confirmation of whether your
credentials are valid and have trading permissions.
