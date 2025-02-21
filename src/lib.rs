//src/lib.rs
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use stylus_sdk::{alloy_primitives::{Address, U256, FixedBytes}, prelude::*, msg};
use stylus_sdk::alloy_sol_types::sol;
use stylus_sdk::storage::{StorageArray, StorageAddress, StorageU256};

// Game board is 3x3
const BOARD_SIZE: usize = 9;

sol! {
    event GameStarted(address indexed player);
    event PlayerMove(uint256 position);
    event ContractMove(uint256 position);
    event GameWon(address indexed winner);
    event GameDrawn();
}

#[entrypoint]
#[storage]
pub struct Contract {
    // The game board (0 = empty, 1 = player, 2 = contract)
    board: StorageArray<StorageU256, BOARD_SIZE>,
    // Player address
    player: StorageAddress,
    // Current turn (1 = player's turn, 2 = contract's turn)
    current_turn: StorageU256,
    // Game status (0 = not started, 1 = in progress, 2 = finished)
    game_status: StorageU256,
    // RNG seed for contract moves
    rng_seed: StorageU256
}

#[public]
impl Contract {
    pub fn constructor(&mut self) {
        self.game_status.set(U256::from(0));
        self.rng_seed.set(U256::from(1));
    }

    pub fn supports_interface(&self, interface: FixedBytes<4>) -> bool {
        let interface_slice_array: [u8; 4] = interface.as_slice().try_into().unwrap();
        let id = u32::from_be_bytes(interface_slice_array);
        
        id == 0x01ffc9a7 // ERC-165
    }

    // Start a new game
    pub fn start_game(&mut self) -> Result<(), Vec<u8>> {
        if self.game_status.get() != U256::from(0) {
            return Err("Game already in progress".as_bytes().to_vec());
        }

        // Get caller using msg::sender()
        let player = msg::sender();

        // Initialize the board
        for i in 0..BOARD_SIZE {
            self.board.setter(i).unwrap().set(U256::from(0));
        }

        self.player.set(player);
        self.current_turn.set(U256::from(1)); // Player goes first
        self.game_status.set(U256::from(1)); // Game in progress

        // Pass VM context to log function
        log(self.vm(), GameStarted { player });
        Ok(())
    }

    // Player makes a move
    pub fn make_move(&mut self, position: U256) -> Result<(), Vec<u8>> {
        let pos = position.try_into().unwrap_or(BOARD_SIZE);
        if pos >= BOARD_SIZE {
            return Err("Invalid position".as_bytes().to_vec());
        }

        if self.game_status.get() != U256::from(1) {
            return Err("Game not in progress".as_bytes().to_vec());
        }

        let player = msg::sender();
        if player != self.player.get() {
            return Err("Not your game".as_bytes().to_vec());
        }

        if self.current_turn.get() != U256::from(1) {
            return Err("Not your turn".as_bytes().to_vec());
        }

        // Check if position is empty
        if self.board.get(pos).unwrap() != U256::from(0) {
            return Err("Position already taken".as_bytes().to_vec());
        }

        // Make the player's move
        self.board.setter(pos).unwrap().set(U256::from(1));
        log(self.vm(), PlayerMove { position });

        // Check for win
        if self.check_winner() {
            self.game_status.set(U256::from(2));
            log(self.vm(), GameWon { winner: player });
            return Ok(());
        }

        // Check for draw
        if self.is_board_full() {
            self.game_status.set(U256::from(2));
            log(self.vm(), GameDrawn {});
            return Ok(());
        }

        // Contract's turn
        self.current_turn.set(U256::from(2));
        
        // Make contract's move
        self.make_contract_move();

        Ok(())
    }

    // Get the current game state
    pub fn get_game_state(&self) -> ([U256; BOARD_SIZE], Address, U256, U256) {
        let mut board = [U256::from(0); BOARD_SIZE];
        for i in 0..BOARD_SIZE {
            board[i] = self.board.get(i).unwrap();
        }
        (
            board,
            self.player.get(),
            self.current_turn.get(),
            self.game_status.get()
        )
    }
}

impl Contract {
    // Contract makes its move using simple strategy
    fn make_contract_move(&mut self) {
        // 1. Try to win
        if let Some(pos) = self.find_winning_move(U256::from(2)) {
            self.make_contract_move_at(pos);
            return;
        }

        // 2. Block player's winning move
        if let Some(pos) = self.find_winning_move(U256::from(1)) {
            self.make_contract_move_at(pos);
            return;
        }

        // 3. Take center if available
        if self.board.get(4).unwrap() == U256::from(0) {
            self.make_contract_move_at(4);
            return;
        }

        // 4. Take a corner if available
        let corners = [0, 2, 6, 8];
        for &corner in corners.iter() {
            if self.board.get(corner).unwrap() == U256::from(0) {
                self.make_contract_move_at(corner);
                return;
            }
        }

        // 5. Take any available spot
        for i in 0..BOARD_SIZE {
            if self.board.get(i).unwrap() == U256::from(0) {
                self.make_contract_move_at(i);
                return;
            }
        }
    }

    // Helper to make contract's move at specific position
    fn make_contract_move_at(&mut self, position: usize) {
        self.board.setter(position).unwrap().set(U256::from(2));
        log(self.vm(), ContractMove { position: U256::from(position) });

        // Check if contract won
        if self.check_winner() {
            self.game_status.set(U256::from(2));
            log(self.vm(), GameWon { winner: Address::ZERO }); // Contract's address is ZERO
            return;
        }

        // Check for draw
        if self.is_board_full() {
            self.game_status.set(U256::from(2));
            log(self.vm(), GameDrawn {});
            return;
        }

        // Switch back to player's turn
        self.current_turn.set(U256::from(1));
    }

    // Find a winning move for the given player number
    fn find_winning_move(&self, player: U256) -> Option<usize> {
        // Check each empty position
        for pos in 0..BOARD_SIZE {
            if self.board.get(pos).unwrap() == U256::from(0) {
                // Try the move
                let mut board_copy = [U256::from(0); BOARD_SIZE];
                for i in 0..BOARD_SIZE {
                    board_copy[i] = self.board.get(i).unwrap();
                }
                board_copy[pos] = player;
                
                // Check if this move would win
                if self.would_win(&board_copy) {
                    return Some(pos);
                }
            }
        }
        None
    }

    // Check if this board state is a win
    fn would_win(&self, board: &[U256; BOARD_SIZE]) -> bool {
        // Check rows
        for i in (0..BOARD_SIZE).step_by(3) {
            if board[i] != U256::from(0) &&
               board[i] == board[i + 1] &&
               board[i] == board[i + 2] {
                return true;
            }
        }

        // Check columns
        for i in 0..3 {
            if board[i] != U256::from(0) &&
               board[i] == board[i + 3] &&
               board[i] == board[i + 6] {
                return true;
            }
        }

        // Check diagonals
        if board[0] != U256::from(0) &&
           board[0] == board[4] &&
           board[0] == board[8] {
            return true;
        }

        if board[2] != U256::from(0) &&
           board[2] == board[4] &&
           board[2] == board[6] {
            return true;
        }

        false
    }

    // Check if there's a winner
    fn check_winner(&self) -> bool {
        // Check rows
        for i in (0..BOARD_SIZE).step_by(3) {
            if self.board.get(i).unwrap() != U256::from(0) &&
               self.board.get(i).unwrap() == self.board.get(i + 1).unwrap() &&
               self.board.get(i).unwrap() == self.board.get(i + 2).unwrap() {
                return true;
            }
        }

        // Check columns
        for i in 0..3 {
            if self.board.get(i).unwrap() != U256::from(0) &&
               self.board.get(i).unwrap() == self.board.get(i + 3).unwrap() &&
               self.board.get(i).unwrap() == self.board.get(i + 6).unwrap() {
                return true;
            }
        }

        // Check diagonals
        if self.board.get(0).unwrap() != U256::from(0) &&
           self.board.get(0).unwrap() == self.board.get(4).unwrap() &&
           self.board.get(0).unwrap() == self.board.get(8).unwrap() {
            return true;
        }

        if self.board.get(2).unwrap() != U256::from(0) &&
           self.board.get(2).unwrap() == self.board.get(4).unwrap() &&
           self.board.get(2).unwrap() == self.board.get(6).unwrap() {
            return true;
        }

        false
    }

    // Check if the board is full (draw)
    fn is_board_full(&self) -> bool {
        for i in 0..BOARD_SIZE {
            if self.board.get(i).unwrap() == U256::from(0) {
                return false;
            }
        }
        true
    }
}