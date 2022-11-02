use serde::*;
use borsh::*;
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Date(pub u32);

impl From<Date> for u32{
    fn from(date: Date) -> Self {
        date.0
    }
}
impl From<Date> for i32{
    fn from(date: Date) -> Self {
        date.0 as i32
    }
}

use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(not(target_os = "solana"))] {
        use chrono::prelude::*;

        impl Date{
            pub fn to_ymd(self)->String{
                self.format("%Y-%m-%d")
            }
            pub fn format(self, str:&str)->String{
                let n_date = NaiveDate::from_num_days_from_ce(self.0 as i32);
                n_date.format(str).to_string()
            }
        }
        //use crate::error::Error;
        impl From<DateTime::<Utc>> for Date{
            fn from(dt: DateTime::<Utc>) -> Self {
                Self(dt.num_days_from_ce() as u32)
            }
        }

        impl From<Date> for DateTime::<Utc>{
            fn from(date: Date) -> Self {
                let n_date = NaiveDate::from_num_days_from_ce(date.0 as i32);
                let n_time = NaiveTime::from_hms(0, 0, 0);
                let n_date_time = NaiveDateTime::new(n_date, n_time);
                DateTime::<Utc>::from_utc(n_date_time, Utc)
            }
        }

        impl ToString for Date{
            fn to_string(&self) -> String {
                String::from(self.clone())
            }
        }

        impl From<Date> for String{
            fn from(date: Date) -> Self {
                DateTime::<Utc>::from(date).format("%d/%m/%Y").to_string()
            }
        }

        impl TryFrom<String> for Date{
            type Error = String;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.as_str().try_into()
            }
        }

        impl TryFrom<&str> for Date{
            type Error = String;//Error
            fn try_from(str: &str) -> Result<Self, Self::Error> {
                let parts = str.split("-");
                let mut ymd = Vec::new();
                for v in parts{
                    if let Ok(v) = v.parse::<u32>(){
                        ymd.push(v)
                    }
                }

                let err = format!("Unable to parse date `{}`", str);

                if ymd.len() < 3{
                    return Err(err);
                }

                let d = NaiveDate::from_ymd(ymd[0] as i32, ymd[1], ymd[2]);
                let t = NaiveTime::from_hms_milli(0,0,0,0);
                
                let ndt = NaiveDateTime::new(d, t);
                Ok(DateTime::<Utc>::from_utc(ndt, Utc).into())
            }
        }
    }
}
