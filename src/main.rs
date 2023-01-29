#![allow(non_snake_case)]
#![allow(dead_code)]

use std::collections::HashMap;

use aws_config;
use aws_sdk_dynamodb::{
    self,
    model::{
        AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
        ScalarAttributeType,
    },
    output::CreateTableOutput,
    Error,
};

use aws_sdk_dynamodb::output::DeleteItemOutput;

use database::Smlr::WorkerInfo;

enum WorkInfoEnum {
    GroundNum = 1,
    HelmetNum,
    Spo2Level,
    Temperature,
    GasLevel,
    HeartRate,
}

async fn InsertItemIntoTable(
    client: &aws_sdk_dynamodb::Client,
    table: &str,
    primKey: &str,
    workInfo: &WorkerInfo,
) -> Result<bool, Error> {
    let request = client
        .put_item()
        .table_name(table)
        .item("GroundNum", AttributeValue::S(workInfo.GroundNum.clone()))
        .item("HelmetNum", AttributeValue::S(workInfo.HelmetNum.clone()))
        .item(
            "Spo2Level",
            AttributeValue::N(workInfo.Spo2Level.to_string()),
        )
        .item(
            "Temperature",
            AttributeValue::N(workInfo.Temperature.to_string()),
        )
        .item(
            "HeartRate",
            AttributeValue::N(workInfo.HeartRate.to_string()),
        )
        .item("GasLevel", AttributeValue::N(workInfo.GasLevel.to_string()))
        .item(
            primKey,
            AttributeValue::S(GetUniqueId(&workInfo.GroundNum, &workInfo.HelmetNum)),
        );

    let output = request.send().await;

    match output {
        Ok(_output) => {
            println!("Value Inserted");
            Ok(true)
        }
        Err(err) => Err(err.into()),
    }
}

async fn CheckTableExist(client: &aws_sdk_dynamodb::Client, table: &str) -> Result<bool, Error> {
    let table_list = client.list_tables().send().await;

    match table_list {
        Ok(list) => Ok(list.table_names().as_ref().unwrap().contains(&table.into())),
        Err(e) => Err(e.into()),
    }
}

async fn DeleteItemFromTable(
    client: &aws_sdk_dynamodb::Client,
    table: &str,
    key: &str,
    value: &str,
) -> Result<DeleteItemOutput, Error> {
    match client
        .delete_item()
        .table_name(table)
        .key(key, AttributeValue::S(value.into()))
        .send()
        .await
    {
        Ok(out) => {
            println!("Deleted item from table");
            Ok(out)
        }
        Err(e) => Err(e.into()),
    }
}

// primKey => primKey
async fn CreateTable(
    client: &aws_sdk_dynamodb::Client,
    table: &str,
    primKey: &str,
) -> Result<CreateTableOutput, Error> {
    let a_name: String = primKey.into();
    let table_name: String = table.into();

    //primKey
    let ad1 = AttributeDefinition::builder()
        .attribute_name(&a_name)
        .attribute_type(ScalarAttributeType::S)
        .build();

    //GroundNum
    let ad2 = AttributeDefinition::builder()
        .attribute_name("GroundNum")
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks1 = KeySchemaElement::builder()
        .attribute_name(&a_name)
        .key_type(KeyType::Hash)
        .build();

    let ks2 = KeySchemaElement::builder()
        .attribute_name("GroundNum")
        .key_type(KeyType::Range)
        .build();

    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build();

    let create_table_response = client
        .create_table()
        .table_name(table_name)
        .key_schema(ks1)
        .attribute_definitions(ad1)
        .attribute_definitions(ad2)
        .key_schema(ks2)
        .provisioned_throughput(pt)
        .send()
        .await;

    match create_table_response {
        Ok(out) => {
            println!("Added table {} with key {}", table, primKey);
            Ok(out)
        }
        Err(e) => {
            eprintln!("Got an error creating table:");
            Err(e.into())
        }
    }
}

async fn DeleteTable(client: &aws_sdk_dynamodb::Client, table_name: &str) -> Result<bool, Error> {
    let response = client.delete_table().table_name(table_name).send().await;

    match response {
        Ok(_output) => Ok(true),
        Err(err) => Err(err.into()),
    }
}

async fn GetAllItemsFromTable(
    client: &aws_sdk_dynamodb::Client,
    table: &str,
) -> Result<Vec<WorkerInfo>, Error> {
    let items = client.scan().table_name(table).send().await?;
    let mut res: Vec<WorkerInfo> = Vec::new();

    if let Some(items) = items.items {
        for item in items {
            res.push(GetWorkInfo(item));
        }
    }

    Ok(res)
}

pub async fn QueryItemFromTable(
    client: &aws_sdk_dynamodb::Client,
    table: &str,
    primKey: &str,
) -> Result<Vec<WorkerInfo>, Error> {
    let results = client
        .query()
        .table_name(table)
        .key_condition_expression("#pk = :pkVal")
        .expression_attribute_names("#pk", "PrimKey")
        .expression_attribute_values(":pkVal", AttributeValue::S(String::from(primKey)))
        .send()
        .await?;
    let mut res: Vec<WorkerInfo> = Vec::new();

    if let Some(items) = results.items {
        for item in items {
            res.push(GetWorkInfo(item));
        }
        Ok(res)
    } else {
        Ok(vec![])
    }
}

fn GetWorkInfo(hashMap: HashMap<String, AttributeValue>) -> WorkerInfo {
    let mut resWork: WorkerInfo =
        serde_json::from_str::<WorkerInfo>(&WorkerInfo::GetRandData()).unwrap();

    for (index, value) in hashMap {
        if index == "GroundNum" {
            resWork.GroundNum = String::from(value.as_s().unwrap());
        } else if index == "GasLevel" {
            resWork.GasLevel = value.as_n().unwrap().parse::<u16>().unwrap();
        } else if index == "HeartRate" {
            resWork.HeartRate = value.as_n().unwrap().parse::<u8>().unwrap();
        } else if index == "Spo2Level" {
            resWork.Spo2Level = value.as_n().unwrap().parse::<u8>().unwrap();
        } else if index == "Temperature" {
            resWork.Temperature = value.as_n().unwrap().parse::<u16>().unwrap();
        } else {
            resWork.HelmetNum = String::from(value.as_s().unwrap());
        }
    }

    return resWork;
}

pub async fn GetClient() -> aws_sdk_dynamodb::Client {
    let region_provider =
        aws_config::meta::region::RegionProviderChain::default_provider().or_else("ap-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;

    let client = aws_sdk_dynamodb::Client::new(&config);

    return client;
}

fn GetUniqueId(groundNum: &String, helNum: &String) -> String {
    let mut res = String::new();
    res.push_str(&groundNum);
    res.push_str(&helNum);

    return res;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = GetClient().await;

    // let count: i32 = 5;

    // for _ in 1..=count {
    //     let inputFromTcp = Smlr::WorkerInfo::GetRandData();
    //     let mut myWork = serde_json::from_str::<Smlr::WorkerInfo>(&inputFromTcp).unwrap();

    //     println!("{:?} {}", myWork, myWork.GroundNum);
    // }

    let tableName = "MyTableTest1";
    let primKey = "PrimKey";

    // DeleteTable(&client, "MyTestTable").await.unwrap();
    // CreateTable(&client, tableName, primKey).await.unwrap();
    // let inputFromTcp = Smlr::WorkerInfo::GetRandData();
    // let myWork = serde_json::from_str::<Smlr::WorkerInfo>(&inputFromTcp).unwrap();
    // InsertItemIntoTable(&client, tableName, primKey, &myWork)
    //     .await
    //     .unwrap();
    println!(
        "{:?}",
        GetAllItemsFromTable(&client, tableName).await.unwrap()
    );
    Ok(())
}
