use std::collections::HashMap;

use async_trait::async_trait;
use qdrant_client::qdrant::value::Kind;
use qdrant_client::qdrant::{
    self, CountPointsBuilder, CreateCollectionBuilder, DeletePointsBuilder, GetPointsBuilder,
    OptimizersConfigDiff, PointId, PointStruct, PointsIdsList, QueryPointsBuilder,
    UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::{Qdrant, QdrantError};
use serde::Serialize;
use serde_json::json;
use tonic::{Code, Status};

const DEFAULT_SEGMENT_NUMBER: u64 = 1;

pub type Payload = qdrant_client::Payload;

pub type Condition = qdrant::Condition;

pub type MinSholud = qdrant::MinShould;

pub enum Distance {
    Cos,
    Dot,
    Euc,
    Man,
}

impl Distance {
    fn into(self) -> qdrant::Distance {
        match self {
            Self::Cos => qdrant::Distance::Cosine,
            Self::Dot => qdrant::Distance::Dot,
            Self::Euc => qdrant::Distance::Euclid,
            Self::Man => qdrant::Distance::Manhattan,
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Not,
    F64(f64),
    I64(i64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Hash(HashMap<String, Value>),
}

impl From<qdrant::Value> for Value {
    fn from(value: qdrant::Value) -> Self {
        match value.kind {
            Some(Kind::DoubleValue(v)) => Value::F64(v),
            Some(Kind::IntegerValue(v)) => Value::I64(v),
            Some(Kind::StringValue(v)) => Value::Str(v),
            Some(Kind::BoolValue(v)) => Value::Bool(v),
            Some(Kind::StructValue(v)) => Value::Hash(
                v.fields
                    .into_iter()
                    .map(|(k, v)| (k, Value::from(v)))
                    .collect(),
            ),
            Some(Kind::ListValue(v)) => {
                Value::List(v.values.into_iter().map(Value::from).collect())
            }
            _ => Value::Not,
        }
    }
}

pub struct Filter(qdrant::Filter);

impl Filter {
    pub fn new() -> Self {
        Self(qdrant::Filter::default())
    }

    pub fn into_inner(self) -> qdrant::Filter {
        self.0
    }

    pub fn must_filter(mut self, must: Vec<Condition>) -> Self {
        self.0.must = must;
        self
    }

    pub fn must_not_filter(mut self, must_not: Vec<Condition>) -> Self {
        self.0.must_not = must_not;
        self
    }

    pub fn should_filter(mut self, should: Vec<Condition>) -> Self {
        self.0.should = should;
        self
    }

    pub fn min_should_filter(mut self, min_should: Option<MinSholud>) -> Self {
        self.0.min_should = min_should;
        self
    }
}

pub struct Point<T: Serialize> {
    pub id: u64,
    pub vector: Vec<f32>,
    pub payload: T,
}

// impl<T: Serialize> Point<T> {
//     pub fn new(id: u64, vector: Vec<f32>, payload: T) -> Self {
//         Self {
//             id,
//             vector,
//             payload,
//         }
//     }
// }

#[async_trait]
pub trait QdrantLibrary {
    // 列出所有集合
    async fn async_list_collections(&self) -> Result<Vec<String>, QdrantError>;

    // 创建一个集合(创建一个已存在的集合就会报错)
    async fn async_create_collection(
        &self,
        collection: String,
        size: u64,
        distance: Distance,
    ) -> Result<(), QdrantError>;

    // 删除一个集合(删除一个不存在的集合不会报错)
    async fn async_delete_collection(&self, collection: String) -> Result<(), QdrantError>;

    // 查看集合信息(集合状态、点数量、向量数量)
    async fn async_collection_info(
        &self,
        collection: String,
    ) -> Result<(i32, u64, u64), QdrantError>;

    // 统计一个集合中有多少个点
    async fn async_count(&self, collection: String, filter: Filter) -> Result<u64, QdrantError>;

    // 批量获取负载
    async fn async_get_points_with_payload(
        &self,
        collection: String,
        ids: Vec<u64>,
    ) -> Result<Vec<HashMap<String, Value>>, QdrantError>;

    // 批量插入点、批量更新点
    async fn async_upsert_points<T: Serialize + Send>(
        &self,
        collection: String,
        points: Vec<Point<T>>,
    ) -> Result<(), QdrantError>;

    // 根据一组特定ID删除点
    // 删除其中一个不存在的点不会发生错误
    async fn async_delete_points(
        &self,
        collection: String,
        ids: Vec<u64>,
    ) -> Result<(), QdrantError>;

    // 根据过滤条件来删除点
    async fn async_delete_points_with_filter(
        &self,
        collection: String,
        filter: Filter,
    ) -> Result<(), QdrantError>;

    // 给定一个输入向量, 搜索与之相似的点, 返回相似度与属性的元组集合
    async fn async_search_points(
        &self,
        collection: String,
        vector: Vec<f32>,
        limit: u64,
        offset: u64,
        score_threshold: f32,
        filter: Filter,
    ) -> Result<Vec<(f32, HashMap<String, Value>)>, QdrantError>;
}

#[async_trait]
impl QdrantLibrary for Qdrant {
    async fn async_list_collections(&self) -> Result<Vec<String>, QdrantError> {
        self.list_collections().await.map(|response| {
            response
                .collections
                .into_iter()
                .map(|description| description.name)
                .collect::<Vec<_>>()
        })
    }

    async fn async_create_collection(
        &self,
        collection: String,
        size: u64,
        distance: Distance,
    ) -> Result<(), QdrantError> {
        self.create_collection(
            CreateCollectionBuilder::new(collection)
                .vectors_config(VectorParamsBuilder::new(size, distance.into()))
                .optimizers_config(OptimizersConfigDiff {
                    default_segment_number: Some(DEFAULT_SEGMENT_NUMBER),
                    ..Default::default()
                }),
        )
        .await
        .map(|_| ())
    }

    async fn async_delete_collection(&self, collection: String) -> Result<(), QdrantError> {
        self.delete_collection(collection).await.map(|_| ())
    }

    async fn async_collection_info(
        &self,
        collection: String,
    ) -> Result<(i32, u64, u64), QdrantError> {
        self.collection_info(collection)
            .await?
            .result
            .map(|info| {
                (
                    info.status().into(),
                    info.points_count(),
                    info.vectors_count(),
                )
            })
            .ok_or_else(|| QdrantError::ResponseError {
                status: Status::new(Code::Internal, "collection info not found"),
            })
    }

    async fn async_count(&self, collection: String, filter: Filter) -> Result<u64, QdrantError> {
        self.count(
            CountPointsBuilder::new(collection)
                .filter(filter.into_inner())
                .exact(true),
        )
        .await
        .map_err(QdrantError::from)?
        .result
        .ok_or_else(|| QdrantError::ResponseError {
            status: Status::new(Code::Internal, "count result is missing"),
        })
        .map(|result| result.count)
    }

    async fn async_get_points_with_payload(
        &self,
        collection: String,
        ids: Vec<u64>,
    ) -> Result<Vec<HashMap<String, Value>>, QdrantError> {
        self.get_points(
            GetPointsBuilder::new(
                collection,
                ids.into_iter().map(PointId::from).collect::<Vec<_>>(),
            )
            .with_payload(true),
        )
        .await
        .map(|r| {
            r.result
                .into_iter()
                .map(|p| {
                    p.payload
                        .into_iter()
                        .map(|(key, val)| (key, Value::from(val)))
                        .collect()
                })
                .collect::<Vec<_>>()
        })
    }

    async fn async_upsert_points<T: Serialize + Send>(
        &self,
        collection: String,
        points: Vec<Point<T>>,
    ) -> Result<(), QdrantError> {
        let points = points
            .into_iter()
            .map(|point| {
                Payload::try_from(json!(point.payload))
                    .map(|payload| PointStruct::new(point.id, point.vector, payload))
                    .map_err(QdrantError::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.upsert_points(UpsertPointsBuilder::new(collection, points).wait(false))
            .await
            .map(|_| ())
    }

    async fn async_delete_points(
        &self,
        collection: String,
        ids: Vec<u64>,
    ) -> Result<(), QdrantError> {
        self.delete_points(
            DeletePointsBuilder::new(collection)
                .points(PointsIdsList {
                    ids: ids.into_iter().map(Into::into).collect(),
                })
                .wait(false),
        )
        .await
        .map(|_| ())
    }

    async fn async_delete_points_with_filter(
        &self,
        collection: String,
        filter: Filter,
    ) -> Result<(), QdrantError> {
        self.delete_points(
            DeletePointsBuilder::new(collection)
                .points(filter.into_inner())
                .wait(false),
        )
        .await
        .map(|_| ())
    }

    async fn async_search_points(
        &self,
        collection: String,
        vector: Vec<f32>,
        limit: u64,
        offset: u64,
        score_threshold: f32,
        filter: Filter,
    ) -> Result<Vec<(f32, HashMap<String, Value>)>, QdrantError> {
        let points = self
            .query(
                QueryPointsBuilder::new(collection)
                    .query(vector)
                    .limit(limit)
                    .offset(offset)
                    .with_payload(true)
                    .score_threshold(score_threshold)
                    .filter(filter.into_inner()),
            )
            .await?
            .result
            .into_iter()
            .map(|r| {
                (
                    r.score,
                    r.payload
                        .into_iter()
                        .map(|(key, val)| (key, Value::from(val)))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .collect::<Vec<_>>();
        Ok(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_async_list_collections() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        if let Ok(collections) = qdrant.async_list_collections().await {
            println!("{:?}", collections);
        }
    }

    #[tokio::test]
    async fn mock_async_create_collection() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        if let Err(error) = qdrant
            .async_create_collection("rust".to_owned(), 768, Distance::Cos)
            .await
        {
            eprintln!("{:?}", error);
        }
    }

    #[tokio::test]
    async fn mock_async_delete_collection() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        if let Err(error) = qdrant.async_delete_collection("rust".to_owned()).await {
            eprintln!("{:?}", error);
        }
    }

    #[tokio::test]
    async fn mock_async_collection_info() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        if let Ok((status, point, vector)) =
            qdrant.async_collection_info("thinktank".to_owned()).await
        {
            println!("{status} {point} {vector}")
        }
    }

    #[tokio::test]
    async fn mock_async_count() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        if let Ok(counts) = qdrant
            .async_count("thinktank".to_owned(), Filter::new())
            .await
        {
            println!("{:?}", counts)
        }
    }

    #[tokio::test]
    async fn async_upsert_points() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        let points = (0..100)
            .map(|id| Point {
                id,
                vector: [0.1; 768].into(),
                payload: json!({
                    "source": format!("source{}", id),
                    "string": format!("string{}", id),
                }),
            })
            .collect::<Vec<_>>();
        if let Err(error) = qdrant.async_upsert_points("rust".to_owned(), points).await {
            eprintln!("{error}")
        }
    }

    #[tokio::test]
    async fn mock_async_get_payloads() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        if let Ok(payloads) = qdrant
            .async_get_points_with_payload("rust".to_owned(), vec![1])
            .await
        {
            payloads
                .into_iter()
                .for_each(|payload| println!("{:?}", payload));
        }
    }

    #[tokio::test]
    async fn mock_async_delete_points() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        if let Err(error) = qdrant.async_delete_points("rust".to_owned(), vec![3]).await {
            eprintln!("{}", error)
        }
    }

    #[tokio::test]
    async fn mock_async_delete_points_with_filter() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        let mut filter = Filter::new();
        filter = filter.must_filter(vec![Condition::matches("source", "source11".to_string())]);
        if let Err(error) = qdrant
            .async_delete_points_with_filter("rust".into(), filter)
            .await
        {
            eprintln!("{error}")
        }
    }

    #[tokio::test]
    async fn mock_async_search_points() {
        let qdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();
        if let Ok(points) = qdrant
            .async_search_points(
                "thinktank".into(),
                [0.1; 768].into(),
                5,
                20,
                0.0,
                Filter::new(),
            )
            .await
        {
            points.into_iter().for_each(|point| println!("{:?}", point));
        }
    }
}
