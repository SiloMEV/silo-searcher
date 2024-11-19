```plaintext   
███████╗██╗██╗      ██████╗ 
██╔════╝██║██║     ██╔═══██╗
███████╗██║██║     ██║   ██║
╚════██║██║██║     ██║   ██║
███████║██║███████╗╚██████╔╝
╚══════╝╚═╝╚══════╝ ╚═════╝                                                                                                                                                                                                  
  ````  
```plaintext   
███████╗███████╗ █████╗ ██████╗  ██████╗██╗  ██╗███████╗██████╗ 
██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██║  ██║██╔════╝██╔══██╗
███████╗█████╗  ███████║██████╔╝██║     ███████║█████╗  ██████╔╝
╚════██║██╔══╝  ██╔══██║██╔══██╗██║     ██╔══██║██╔══╝  ██╔══██╗
███████║███████╗██║  ██║██║  ██║╚██████╗██║  ██║███████╗██║  ██║
╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝                                                                                                                                                                                                                                    
````
--- 
Howdy Farmors! Time to cultivate some DEX pools!

Welcome! This repository provides example code to execute basic decentralized finance (DeFi) transactions on an Ethereum Virtual Machine (EVM) chain, such as Sei, and showcases minimal implementations of concepts related to Miner Extractable Value (MEV). Here, you'll find examples for swaps on both Uniswap V2 and V3 style pools, which can serve as starting points for exploring arbitrage opportunities, liquidity pooling, and more complex DeFi strategies.

## Features

- Basic transaction examples on Uniswap V2 and V3 pools
- Swapping tokens over different pool versions
- Starting point for expanding into strategies like:
  - Triangular Arbitrage
  - Pure Arbitrage
  - Centralized Exchange (CEX) Arbitrage


## Setup & Installation

1. Clone this repository:

```bash
git clone https://github.com/SiloMEV/silo-searcher
cd silo-searcher
```

2. Ensure you have Rust installed. If not, install Rust.
3. Run the following commands to build and execute examples.
4. Plow the fields and reap your harvest.

## Usage

### Run Uniswap V2 Pool Swap Example

Simulates a token swap in a Uniswap V2 pool. This example focuses on a single swap and does not yet incorporate full arbitrage analysis. However, it can be extended to simulate multiple pools and calculate potential profits.

```bash
cargo run --example uniswapv2pool_amountout
```

### Run Uniswap V3 Pool Swap Example

Simulates a token swap in a Uniswap V3 pool, demonstrating the added flexibility and fee structure available in V3. This example is a baseline for working with concentrated liquidity pools.

```bash
cargo run --example uniswapv3pool_amountout
```

## Future Expansion Ideas

While the current examples show single pool swaps, you can build on this repository to create more complex strategies and add profit-driven simulations:

Triangular Arbitrage: Calculate opportunities across multiple pairs and pools to find potential profits.\
Pure Arbitrage: Simulate profit opportunities from price discrepancies between pools.\
CEX Arbitrage: Extend this to connect with centralized exchanges for cross-platform arbitrage.

## Get in touch

If you're a searcher and you want to 1) give us feedback 2) flag a bug 3) collab 4) join the team, feel free to reach out to `mev at silostaking.io` 

## License

This project is open-source and available under the MIT License.
