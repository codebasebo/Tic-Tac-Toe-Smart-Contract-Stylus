# Tic-Tac-Toe Smart Contract on Stylus

This repository contains a smart contract implementation of the classic game **Tic-Tac-Toe** using the **Stylus SDK**. The contract allows players to interact with the game on-chain, enabling them to start a new game, make moves, and check the game state. The contract also includes an AI opponent that makes strategic moves to compete against the player.

---

## Table of Contents

- [Tic-Tac-Toe Smart Contract on Stylus](#tic-tac-toe-smart-contract-on-stylus)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Starting a Game](#starting-a-game)
    - [Making a Move](#making-a-move)
    - [Checking the Game State](#checking-the-game-state)
  - [Contract Logic](#contract-logic)
    - [Game Board](#game-board)
    - [Game Flow](#game-flow)
    - [AI Strategy](#ai-strategy)
  - [Events](#events)
  - [Testing](#testing)
  - [Contributing](#contributing)

---

## Overview

The **Tic-Tac-Toe** smart contract is designed to run on the **Stylus** framework, which provides tools for building scalable and efficient smart contracts. The game is played on a 3x3 grid where:
- Players are represented by `1`.
- The contract (AI) is represented by `2`.
- Empty cells are represented by `0`.

The contract supports a single-player mode where the user competes against the contract's AI. The AI uses a simple yet effective strategy to ensure competitive gameplay.

---

## Features

- **Player vs. AI**: A single-player mode where the user competes against the contract's AI.
- **Game Events**: Emits events for game actions such as starting a game, making moves, winning, and drawing.
- **State Management**: Tracks the game board, player turns, and game status (not started, in progress, finished).
- **AI Strategy**: Implements a basic AI strategy to make intelligent moves:
  1. Try to win.
  2. Block the player's winning move.
  3. Take the center if available.
  4. Take a corner if available.
  5. Take any available spot.
- **Error Handling**: Ensures invalid moves or states are handled gracefully with descriptive error messages.

---

## Installation

To use this contract, you need to have the following installed:

1. **Rust**: Install Rust from [rust-lang.org](https://www.rust-lang.org/tools/install).
2. **Stylus SDK**: Follow the instructions in the [Stylus documentation](https://stylus-sdk.alchemy.com/) to set up the SDK.

Clone this repository and navigate to the project directory:

```bash
git clone https://github.com/your-repo/tic-tac-toe-stylus.git
cd tic-tac-toe-stylus
```

Install dependencies:

```bash
cargo build
```

---

## Usage

### Starting a Game

To start a new game, call the `start_game` function. This initializes the game board and sets the player's turn.

```rust
let result = contract.start_game();
assert!(result.is_ok());
```

If successful, the contract emits the `GameStarted` event.

---

### Making a Move

Players can make a move by calling the `make_move` function with the desired position (0-8). Positions are indexed as follows:

```
0 | 1 | 2
---------
3 | 4 | 5
---------
6 | 7 | 8
```

Example:

```rust
let position = U256::from(4); // Center of the board
let result = contract.make_move(position);
assert!(result.is_ok());
```

After the player's move, the contract automatically makes its move if the game is still in progress.

---

### Checking the Game State

To retrieve the current state of the game, call the `get_game_state` function. It returns:
- The game board as an array of `U256` values.
- The player's address.
- The current turn (`1` for the player, `2` for the contract).
- The game status (`0` = not started, `1` = in progress, `2` = finished).

Example:

```rust
let (board, player, current_turn, game_status) = contract.get_game_state();
println!("Board: {:?}", board);
println!("Player: {:?}", player);
println!("Current Turn: {:?}", current_turn);
println!("Game Status: {:?}", game_status);
```

---

## Contract Logic

### Game Board

The game board is represented as a 1D array of size 9 (`BOARD_SIZE`). Each cell can hold one of three values:
- `0`: Empty cell.
- `1`: Player's move.
- `2`: Contract's move.

### Game Flow

1. **Initialization**: The game starts with an empty board and the player's turn.
2. **Player's Turn**: The player makes a move by selecting an empty cell.
3. **AI's Turn**: After the player's move, the contract evaluates the board and makes its move based on the predefined strategy.
4. **Win/Draw Check**: After each move, the contract checks for a winner or a draw. If either condition is met, the game ends.

### AI Strategy

The AI follows a priority-based strategy:
1. **Win**: If the AI can win in the next move, it takes that move.
2. **Block**: If the player can win in the next move, the AI blocks that move.
3. **Center**: If the center cell is empty, the AI takes it.
4. **Corners**: If any corner cell is empty, the AI takes one.
5. **Random**: If no other options are available, the AI takes any empty cell.

---

## Events

The contract emits the following events during gameplay:

- `GameStarted(address indexed player)`: Emitted when a new game starts.
- `PlayerMove(uint256 position)`: Emitted when the player makes a move.
- `ContractMove(uint256 position)`: Emitted when the contract makes a move.
- `GameWon(address indexed winner)`: Emitted when a player wins the game.
- `GameDrawn()`: Emitted when the game ends in a draw.

---

## Testing

To test the contract, run the following command:

```bash
cargo test
```

Tests include:
- Starting a new game.
- Making valid and invalid moves.
- Checking the AI's strategy.
- Verifying win and draw conditions.

---

## Contributing

Contributions are welcome! To contribute:
1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Submit a pull request with a detailed description of your changes.

