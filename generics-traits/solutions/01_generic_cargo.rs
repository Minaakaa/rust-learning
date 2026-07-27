//! 問題 01 の解答例

#[derive(Debug, PartialEq, Eq)]
struct Cargo<T> {
    id: String,
    payload: T,
}

impl<T> Cargo<T> {
    fn new(id: &str, payload: T) -> Self {
        Self {
            id: id.to_string(),
            payload,
        }
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn payload(&self) -> &T {
        &self.payload
    }

    fn into_parts(self) -> (String, T) {
        (self.id, self.payload)
    }

    fn replace<U>(self, new_payload: U) -> (T, Cargo<U>) {
        let Self { id, payload } = self;
        (
            payload,
            Cargo {
                id,
                payload: new_payload,
            },
        )
    }
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
