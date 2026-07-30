#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 04: RefCell で共有参照から安全に状態を更新する
//!
//! 複数の画面から共有される `RobotConsole` に、配送完了を記録します
//! 呼び出し側には `&RobotConsole` だけを渡し、内部の `RefCell<RobotState>` が
//! 借用規則を実行時に検査します
//!
//! 仕様:
//! - `state` は `RobotState` の読み取りガード `Ref<'_, RobotState>` を返す
//! - `try_complete` は `try_borrow_mut` を使い、借用競合で panic せず `Busy` を返す
//! - 電池不足、完了数オーバーフローの順で検証し、失敗時は状態を変更しない
//! - 成功時だけ電池を減らし、完了数を1増やす
//! - 成功時は `Mission` の ID を複製せず `Receipt` へ移す
//! - 失敗時は元の `Mission` をエラーから回収できるようにする
//!
//! エラー優先順位:
//! 1. 借用競合
//! 2. 電池不足
//! 3. 完了数オーバーフロー

use std::cell::{Ref, RefCell};

#[derive(Debug, PartialEq, Eq)]
struct Mission {
    id: String,
    energy_wh: u32,
}

impl Mission {
    fn new(id: &str, energy_wh: u32) -> Self {
        Self {
            id: id.to_owned(),
            energy_wh,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    const fn energy_wh(&self) -> u32 {
        self.energy_wh
    }

    fn into_id(self) -> String {
        self.id
    }
}

#[derive(Debug, PartialEq, Eq)]
struct RobotState {
    battery_wh: u32,
    completed: u32,
}

impl RobotState {
    const fn new(battery_wh: u32, completed: u32) -> Self {
        Self {
            battery_wh,
            completed,
        }
    }

    const fn battery_wh(&self) -> u32 {
        self.battery_wh
    }

    const fn completed(&self) -> u32 {
        self.completed
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Receipt {
    mission_id: String,
    remaining_wh: u32,
    completed: u32,
}

impl Receipt {
    fn mission_id(&self) -> &str {
        &self.mission_id
    }

    const fn remaining_wh(&self) -> u32 {
        self.remaining_wh
    }

    const fn completed(&self) -> u32 {
        self.completed
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CompleteError {
    Busy(Mission),
    InsufficientBattery { mission: Mission, available_wh: u32 },
    CountOverflow(Mission),
}

struct RobotConsole {
    state: RefCell<RobotState>,
}

impl RobotConsole {
    fn new(battery_wh: u32, completed: u32) -> Self {
        Self {
            state: RefCell::new(RobotState::new(battery_wh, completed)),
        }
    }

    fn state(&self) -> Ref<'_, RobotState> {
        todo!("RefCell 内の RobotState を借用する Ref を返してください")
    }

    fn try_complete(&self, mission: Mission) -> Result<Receipt, CompleteError> {
        todo!(
            "共有コンソールから任務 {} の完了を安全に記録してください",
            mission.id()
        )
    }
}

fn main() {
    let console = RobotConsole::new(20, 0);
    let receipt = console
        .try_complete(Mission::new("配送-801", 7))
        .expect("配送を完了できる");

    println!(
        "完了: {}、残量: {} Wh、累計: {} 件",
        receipt.mission_id(),
        receipt.remaining_wh(),
        receipt.completed()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 共有参照から配送完了を記録する() {
        let console = RobotConsole::new(20, 3);

        let receipt = console
            .try_complete(Mission::new("M-801", 7))
            .expect("配送を完了できる");

        assert_eq!(receipt.mission_id(), "M-801");
        assert_eq!(receipt.remaining_wh(), 13);
        assert_eq!(receipt.completed(), 4);
        assert_eq!(console.state().battery_wh(), 13);
        assert_eq!(console.state().completed(), 4);
    }

    #[test]
    fn 電池残量ちょうどの任務を完了する() {
        let console = RobotConsole::new(8, 0);

        let receipt = console
            .try_complete(Mission::new("M-802", 8))
            .expect("残量ちょうどで完了できる");

        assert_eq!(receipt.remaining_wh(), 0);
        assert_eq!(console.state().battery_wh(), 0);
    }

    #[test]
    fn 残量ゼロでも消費ゼロの任務を完了する() {
        let console = RobotConsole::new(0, 9);

        let receipt = console
            .try_complete(Mission::new("M-803", 0))
            .expect("消費ゼロなら完了できる");

        assert_eq!(receipt.remaining_wh(), 0);
        assert_eq!(receipt.completed(), 10);
    }

    #[test]
    fn 電池不足ではmissionを返し状態を変更しない() {
        let console = RobotConsole::new(4, 2);
        let mission = Mission::new("M-804", 5);
        let id_pointer = mission.id().as_ptr();

        let error = console.try_complete(mission).unwrap_err();

        match error {
            CompleteError::InsufficientBattery {
                mission,
                available_wh,
            } => {
                assert_eq!(mission.id(), "M-804");
                assert_eq!(mission.id().as_ptr(), id_pointer);
                assert_eq!(mission.energy_wh(), 5);
                assert_eq!(available_wh, 4);
            }
            other => panic!("想定外のエラー: {other:?}"),
        }
        assert_eq!(*console.state(), RobotState::new(4, 2));
    }

    #[test]
    fn 完了数のオーバーフローではmissionを返し状態を変更しない() {
        let console = RobotConsole::new(10, u32::MAX);
        let mission = Mission::new("M-805", 3);

        let error = console.try_complete(mission).unwrap_err();

        assert_eq!(
            error,
            CompleteError::CountOverflow(Mission::new("M-805", 3))
        );
        assert_eq!(*console.state(), RobotState::new(10, u32::MAX));
    }

    #[test]
    fn 電池不足を完了数オーバーフローより先に返す() {
        let console = RobotConsole::new(0, u32::MAX);

        let error = console.try_complete(Mission::new("M-806", 1)).unwrap_err();

        assert_eq!(
            error,
            CompleteError::InsufficientBattery {
                mission: Mission::new("M-806", 1),
                available_wh: 0,
            }
        );
        assert_eq!(*console.state(), RobotState::new(0, u32::MAX));
    }

    #[test]
    fn 読み取り中はpanicせずbusyでmissionを返す() {
        let console = RobotConsole::new(12, 0);
        let reading = console.state();
        let mission = Mission::new("M-807", 2);
        let id_pointer = mission.id().as_ptr();

        let returned = match console.try_complete(mission).unwrap_err() {
            CompleteError::Busy(mission) => mission,
            other => panic!("想定外のエラー: {other:?}"),
        };

        assert_eq!(returned.id().as_ptr(), id_pointer);
        assert_eq!(reading.battery_wh(), 12);
        drop(reading);

        let receipt = console
            .try_complete(returned)
            .expect("ガード解放後は完了できる");
        assert_eq!(receipt.remaining_wh(), 10);
    }

    #[test]
    fn 可変借用中もpanicせずbusyを返す() {
        let console = RobotConsole::new(9, 1);
        let mutating = console.state.borrow_mut();

        let error = console.try_complete(Mission::new("M-808", 1)).unwrap_err();

        assert_eq!(error, CompleteError::Busy(Mission::new("M-808", 1)));
        assert_eq!(mutating.battery_wh(), 9);
    }

    #[test]
    fn mission_idのstringを複製せずreceiptへ移す() {
        let console = RobotConsole::new(30, 0);
        let mission = Mission::new("配送任務🚚-八", 4);
        let id_pointer = mission.id().as_ptr();

        let receipt = console.try_complete(mission).expect("配送を完了できる");

        assert_eq!(receipt.mission_id(), "配送任務🚚-八");
        assert_eq!(receipt.mission_id().as_ptr(), id_pointer);
    }

    #[test]
    fn 連続する配送で状態を累積する() {
        let console = RobotConsole::new(15, 5);

        let first = console
            .try_complete(Mission::new("第一便", 4))
            .expect("第一便を完了できる");
        let second = console
            .try_complete(Mission::new("第二便🤖", 6))
            .expect("第二便を完了できる");

        assert_eq!((first.remaining_wh(), first.completed()), (11, 6));
        assert_eq!((second.remaining_wh(), second.completed()), (5, 7));
        assert_eq!(*console.state(), RobotState::new(5, 7));
    }
}
