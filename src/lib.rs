#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod Smlr {
    use serde::{Deserialize, Serialize};

    use super::RandData;
    use serde_json::{json, Value};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct WorkerInfo {
        pub GroundNum: String,
        pub HelmetNum: String,
        pub Spo2Level: u8,
        pub Temperature: u16,
        pub GasLevel: u16,
        pub HeartRate: u8,
    }

    impl WorkerInfo {

        pub fn GetRandData() -> String {
            return json!({
                "GroundNum" : RandData::GetRandComb(String::from("001"), vec![3,3,4]),
                "HelmetNum": RandData::GetRandComb(String::from("1"), vec![4]),
                "Spo2Level": RandData::GetRandInt(0,255),
                "Temperature": RandData::GetRandInt(0,255),
                "GasLevel": RandData::GetRandInt(0, 65000),
                "HeartRate": RandData::GetRandInt(0,255)
            })
            .to_string();
        }
    }

    impl WorkerInfo {
        pub fn ToJSON(&self) -> Value {
            return json!(
                {
                    "GroundNum" : self.GroundNum,
                    "HelmetNum": self.HelmetNum,
                    "Spo2Level": self.Spo2Level,
                    "Temperature": self.Temperature,
                    "GasLevel": self.GasLevel,
                    "HeartRate": self.HeartRate
                }
            );
        }
    }
}

pub mod RandData {
    use rand::{self, Rng};
    use serde_json::{json, Value};

    //Only String => Only Caps ( 65-91 )
    pub fn GetRandString(count: i32) -> String {
        let mut res: Vec<char> = Vec::new();

        for _ in 1..=count {
            res.push(char::from_u32(GetRandInt(65, 90) as u32).unwrap_or_default());
        }

        return String::from_iter(res);
    }

    pub fn GetRandVecInt(count: i32) -> Vec<i32> {
        let mut res: Vec<i32> = Vec::new();

        for _ in 1..=count {
            res.push(GetRandInt(0, 9));
        }

        return res;
    }

    /// For all uses this method but then later cast to required type
    ///
    pub fn GetRandInt(Min: i32, Max: i32) -> i32 {
        let mut rng = rand::thread_rng();
        rng.gen_range(Min..=Max)
    }

    pub fn GetRandVecIntAsString(data: Vec<i32>) -> String {
        let mut res = String::new();

        for i in data {
            let tmp: Vec<char> = format!("{}", i).chars().collect();
            res.push(tmp[0]);
        }

        return res;
    }

    // 0 for String , 1 for Int
    pub fn GetRandComb(comb: String, comLen: Vec<i32>) -> String {
        let mut res = String::new();
        let charVec: Vec<char> = comb.chars().collect();
        let compALen = comb.len();

        for (idx, ele) in charVec.into_iter().enumerate() {
            if ele == '0' {
                res.push_str(&GetRandString(comLen[idx]));
            } else {
                res.push_str(&GetRandVecIntAsString(GetRandVecInt(comLen[idx])));
            }

            if idx != compALen - 1 {
                res.push('_');
            }
        }

        return res;
    }

    // pub fn GenRandJson() -> Value {}
}
