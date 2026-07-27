#![cfg_attr(not(test), allow(dead_code))]

//! # 問題 01: ジェネリックな荷物を作る
//!
//! 配送ロボットは、交換部品、測定値、検査結果など、型の異なる荷物を運びます
//! 荷物ごとに構造体を作り直さず、`Cargo<T>` の `T` を中身の型として使ってください
//!
//! 仕様:
//! - `new` は ID を所有する `String` と payload を保存する
//! - `id` と `payload` は保存した値を借用して返す
//! - `into_parts` は `Cargo<T>` を消費し、ID と payload の所有権を返す
//! - `replace` は `Cargo<T>` を消費し、古い payload と同じ ID を持つ `Cargo<U>` を返す
//! - どのメソッドにも `Clone` などの不要なトレイト境界を付けない
//!
//! ヒント:
//! - すべての `T` で使えるメソッドは `impl<T> Cargo<T>` に書く
//! - `replace` ではメソッドだけで使う新しい型 `U` を宣言する
//! - `self` を分解すると、フィールドを複製せず個別に移動できる

#[derive(Debug, PartialEq, Eq)]
struct Cargo<T> {
    id: String,
    payload: T,
}

impl<T> Cargo<T> {
    fn new(id: &str, payload: T) -> Self {
        let _ = &payload;
        todo!(
            "ID {id:?} と型 {} の payload を保存してください",
            std::any::type_name::<T>()
        )
    }

    fn id(&self) -> &str {
        todo!("荷物 ID {:?} を借用してください", self.id)
    }

    fn payload(&self) -> &T {
        todo!("荷物 {} の payload を借用してください", self.id)
    }

    fn into_parts(self) -> (String, T) {
        todo!("荷物 {} を ID と payload に分解してください", self.id)
    }

    fn replace<U>(self, new_payload: U) -> (T, Cargo<U>) {
        let _ = &new_payload;
        todo!(
            "荷物 {} の payload を型 {} に交換してください",
            self.id,
            std::any::type_name::<U>()
        )
    }
}

fn main() {
    let cargo = Cargo::new("CG-501", String::from("距離センサー"));
    println!("荷物 {}: {:?}", cargo.id(), cargo.payload());

    let (old_payload, inspected) = cargo.replace(true);
    println!("取り出した payload: {old_payload}");
    println!("検査済み荷物: {inspected:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct NonClone(String);

    #[derive(Debug, PartialEq, Eq)]
    enum Inspection {
        Pending,
        Accepted,
    }

    struct OpaquePayload(&'static str);

    struct OtherPayload(u8);

    #[test]
    fn 数値をpayloadとして保存する() {
        let cargo = Cargo::new("CG-501", 12_u16);

        assert_eq!(cargo.id(), "CG-501");
        assert_eq!(cargo.payload(), &12);
    }

    #[test]
    fn 同じcargo型を異なるpayload型で使う() {
        let text = Cargo::new("CG-502", String::from("予備バッテリー"));
        let state = Cargo::new("CG-503", Inspection::Pending);

        assert_eq!(text.payload(), "予備バッテリー");
        assert_eq!(state.payload(), &Inspection::Pending);
    }

    #[test]
    fn cloneできないpayloadを借用する() {
        let cargo = Cargo::new("CG-504", NonClone(String::from("制御基板")));
        let borrowed: &NonClone = cargo.payload();

        assert_eq!(borrowed.0, "制御基板");
        assert_eq!(cargo.id(), "CG-504");
    }

    #[test]
    fn into_partsで両方の所有権を取り出す() {
        let cargo = Cargo::new("CG-505", NonClone(String::from("駆動モーター")));

        let (id, payload) = cargo.into_parts();

        assert_eq!(id, "CG-505");
        assert_eq!(payload, NonClone(String::from("駆動モーター")));
    }

    #[test]
    fn replaceでpayload型を変更し古い値も返す() {
        let cargo = Cargo::new("CG-506", NonClone(String::from("未検査")));

        let (old, inspected): (NonClone, Cargo<Inspection>) = cargo.replace(Inspection::Accepted);

        assert_eq!(old, NonClone(String::from("未検査")));
        assert_eq!(inspected.id(), "CG-506");
        assert_eq!(inspected.payload(), &Inspection::Accepted);
    }

    #[test]
    fn replaceを続けてもidを保つ() {
        let cargo = Cargo::new("CG-507", 3_u8);
        let (count, labeled) = cargo.replace(String::from("交換タイヤ"));
        let (label, weighed) = labeled.replace(2_400_u32);

        assert_eq!(count, 3);
        assert_eq!(label, "交換タイヤ");
        assert_eq!(weighed.id(), "CG-507");
        assert_eq!(weighed.payload(), &2_400);
    }

    #[test]
    fn 日本語と絵文字をそのまま保持する() {
        let cargo = Cargo::new("荷物📦-01", String::from("工学部Ａ棟向けセンサー🔧"));

        assert_eq!(cargo.id(), "荷物📦-01");
        assert_eq!(cargo.payload(), "工学部Ａ棟向けセンサー🔧");
    }

    #[test]
    fn 標準トレイトを持たないpayloadにもすべてのapiを使える() {
        let cargo = Cargo::new("CG-508", OpaquePayload("校正前"));
        assert_eq!(cargo.id(), "CG-508");
        assert_eq!(cargo.payload().0, "校正前");

        let (old, replaced) = cargo.replace(OtherPayload(8));
        assert_eq!(old.0, "校正前");
        assert_eq!(replaced.payload().0, 8);

        let (id, payload) = replaced.into_parts();
        assert_eq!(id, "CG-508");
        assert_eq!(payload.0, 8);
    }
}
