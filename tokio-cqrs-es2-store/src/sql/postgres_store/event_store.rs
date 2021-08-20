use async_trait::async_trait;
use std::{
    collections::HashMap,
    marker::PhantomData,
};

use sqlx::postgres::PgPool;

use cqrs_es2_core::{
    AggregateContext,
    Error,
    EventContext,
    IAggregate,
    ICommand,
    IEvent,
};

use crate::repository::IEventStore;

use super::constants::{
    INSERT_EVENT,
    INSERT_SNAPSHOT,
    SELECT_EVENTS,
    SELECT_EVENTS_WITH_METADATA,
    SELECT_SNAPSHOT,
    UPDATE_SNAPSHOT,
};

/// Storage engine using an Postgres backing and relying on a
/// serialization of the aggregate rather than individual events. This
/// is similar to the "snapshot strategy" seen in many CQRS
/// frameworks.
pub struct EventStore<C: ICommand, E: IEvent, A: IAggregate<C, E>> {
    pool: PgPool,
    with_snapshots: bool,
    _phantom: PhantomData<(C, E, A)>,
}

impl<C: ICommand, E: IEvent, A: IAggregate<C, E>>
    EventStore<C, E, A>
{
    /// Creates a new `EventStore` from the provided
    /// database connection.
    pub fn new(
        pool: PgPool,
        with_snapshots: bool,
    ) -> Self {
        EventStore {
            pool,
            with_snapshots,
            _phantom: PhantomData,
        }
    }

    async fn load_aggregate_from_snapshot(
        &mut self,
        aggregate_id: &str,
    ) -> Result<AggregateContext<C, E, A>, Error> {
        let agg_type = A::aggregate_type();
        let id = aggregate_id.to_string();

        let rows: Vec<(i64, serde_json::Value)> =
            match sqlx::query_as(SELECT_SNAPSHOT)
                .bind(&agg_type)
                .bind(&id)
                .fetch_all(&self.pool)
                .await
            {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "could not load events table for \
                             aggregate id {} with error: {}",
                            &id, e
                        )
                        .as_str(),
                    ))
                },
            };

        if rows.len() == 0 {
            return Ok(AggregateContext::new(
                id,
                A::default(),
                0,
            ));
        };

        let row = rows[0].clone();

        let aggregate = match serde_json::from_value(row.1) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "bad payload found in events table for \
                         aggregate id {} with error: {}",
                        &id, e
                    )
                    .as_str(),
                ));
            },
        };

        Ok(AggregateContext::new(
            id,
            aggregate,
            row.0 as usize,
        ))
    }

    async fn load_aggregate_from_events(
        &mut self,
        aggregate_id: &str,
    ) -> Result<AggregateContext<C, E, A>, Error> {
        let id = aggregate_id.to_string();

        let events = match self.load_events(&id, false).await {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            },
        };

        if events.len() == 0 {
            return Ok(AggregateContext::new(
                id,
                A::default(),
                0,
            ));
        }

        let mut aggregate = A::default();

        events
            .iter()
            .map(|x| &x.payload)
            .for_each(|x| aggregate.apply(&x));

        Ok(AggregateContext::new(
            id,
            aggregate,
            events.last().unwrap().sequence,
        ))
    }

    async fn commit_with_snapshots(
        &mut self,
        events: Vec<E>,
        context: AggregateContext<C, E, A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventContext<C, E>>, Error> {
        let agg_type = A::aggregate_type().to_string();
        let aggregate_id = context.aggregate_id.as_str();
        let current_sequence = context.current_sequence;

        let contexts = self.wrap_events(
            aggregate_id,
            current_sequence,
            events,
            metadata,
        );

        let mut last_sequence = current_sequence as i64;
        let mut updated_aggregate = context.aggregate.clone();

        for context in contexts.clone() {
            let sequence = context.sequence as i64;
            last_sequence = sequence;

            let payload = match serde_json::to_value(&context.payload)
            {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "could not serialize payload for \
                             aggregate id {} with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };

            let metadata =
                match serde_json::to_value(&context.metadata) {
                    Ok(x) => x,
                    Err(e) => {
                        return Err(Error::new(
                            format!(
                                "could not serialize metadata for \
                                 aggregate id {} with error: {}",
                                &aggregate_id, e
                            )
                            .as_str(),
                        ));
                    },
                };

            match sqlx::query(INSERT_EVENT)
                .bind(&agg_type)
                .bind(&aggregate_id)
                .bind(sequence)
                .bind(&payload)
                .bind(&metadata)
                .execute(&self.pool)
                .await
            {
                Ok(x) => {
                    if x.rows_affected() != 1 {
                        return Err(Error::new(
                            format!(
                                "insert new events failed for \
                                 aggregate id {}",
                                &aggregate_id
                            )
                            .as_str(),
                        ));
                    }
                },
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "could not insert new events for \
                             aggregate id {} with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };

            updated_aggregate.apply(&context.payload);
        }

        let aggregate_payload =
            match serde_json::to_value(updated_aggregate) {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "could not serialize aggregate snapshot \
                             for aggregate id {} with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };

        let sql = match context.current_sequence {
            0 => INSERT_SNAPSHOT,
            _ => UPDATE_SNAPSHOT,
        };

        match sqlx::query(sql)
            .bind(&agg_type)
            .bind(&aggregate_id)
            .bind(last_sequence)
            .bind(aggregate_payload)
            .execute(&self.pool)
            .await
        {
            Ok(x) => {
                if x.rows_affected() != 1 {
                    return Err(Error::new(
                        format!(
                            "insert new snapshot failed for \
                             aggregate id {}",
                            &aggregate_id
                        )
                        .as_str(),
                    ));
                }
            },
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "could not insert new snapshot for \
                         aggregate id {} with error: {}",
                        &aggregate_id, e
                    )
                    .as_str(),
                ));
            },
        };

        Ok(contexts)
    }

    async fn commit_events_only(
        &mut self,
        events: Vec<E>,
        context: AggregateContext<C, E, A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventContext<C, E>>, Error> {
        let agg_type = A::aggregate_type().to_string();
        let aggregate_id = context.aggregate_id.as_str();
        let current_sequence = context.current_sequence;

        let contexts = self.wrap_events(
            &aggregate_id,
            current_sequence,
            events,
            metadata,
        );

        for context in &contexts {
            let sequence = context.sequence as i64;

            let payload = match serde_json::to_value(&context.payload)
            {
                Ok(x) => x,
                Err(err) => {
                    return Err(Error::new(
                        format!(
                            "Could not serialize the event payload \
                             for aggregate id {} with error: {}",
                            &aggregate_id, err
                        )
                        .as_str(),
                    ));
                },
            };

            let metadata =
                match serde_json::to_value(&context.metadata) {
                    Ok(x) => x,
                    Err(err) => {
                        return Err(Error::new(
                            format!(
                                "could not serialize the event \
                                 metadata for aggregate id {} with \
                                 error: {}",
                                &aggregate_id, err
                            )
                            .as_str(),
                        ));
                    },
                };

            match sqlx::query(INSERT_EVENT)
                .bind(&agg_type)
                .bind(&aggregate_id)
                .bind(sequence)
                .bind(&payload)
                .bind(&metadata)
                .execute(&self.pool)
                .await
            {
                Ok(x) => {
                    if x.rows_affected() != 1 {
                        return Err(Error::new(
                            format!(
                                "insert new events failed for \
                                 aggregate id {}",
                                &aggregate_id
                            )
                            .as_str(),
                        ));
                    }
                },
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "could not insert new events for \
                             aggregate id {} with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };
        }

        Ok(contexts)
    }

    async fn load_events_only(
        &mut self,
        aggregate_id: &str,
    ) -> Result<Vec<EventContext<C, E>>, Error> {
        let agg_type = A::aggregate_type();

        let rows: Vec<(i64, serde_json::Value)> =
            match sqlx::query_as(SELECT_EVENTS)
                .bind(&agg_type)
                .bind(&aggregate_id)
                .fetch_all(&self.pool)
                .await
            {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "could not load events table for \
                             aggregate id {} with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };

        let mut result = Vec::new();

        for row in rows {
            let payload = match serde_json::from_value(row.1) {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "bad payload found in events table for \
                             aggregate id {} with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };

            result.push(EventContext::new(
                aggregate_id.to_string(),
                row.0 as usize,
                payload,
                Default::default(),
            ));
        }

        Ok(result)
    }

    async fn load_events_with_metadata(
        &mut self,
        aggregate_id: &str,
    ) -> Result<Vec<EventContext<C, E>>, Error> {
        let agg_type = A::aggregate_type();

        let rows: Vec<(
            i64,
            serde_json::Value,
            serde_json::Value,
        )> = match sqlx::query_as(SELECT_EVENTS_WITH_METADATA)
            .bind(&agg_type)
            .bind(&aggregate_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(
                    format!(
                        "could not load events table for aggregate \
                         id {} with error: {}",
                        &aggregate_id, e
                    )
                    .as_str(),
                ));
            },
        };

        let mut result = Vec::new();

        for row in rows {
            let payload = match serde_json::from_value(row.1) {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(
                        format!(
                            "bad payload found in events table for \
                             aggregate id {} with error: {}",
                            &aggregate_id, e
                        )
                        .as_str(),
                    ));
                },
            };

            let metadata = match serde_json::from_value(row.2) {
                Ok(x) => x,
                Err(err) => {
                    return Err(Error::new(
                        format!(
                            "bad metadata found in events table for \
                             aggregate id {} with error: {}",
                            &aggregate_id, err
                        )
                        .as_str(),
                    ));
                },
            };

            result.push(EventContext::new(
                aggregate_id.to_string(),
                row.0 as usize,
                payload,
                metadata,
            ));
        }

        Ok(result)
    }
}

#[async_trait]
impl<C: ICommand, E: IEvent, A: IAggregate<C, E>> IEventStore<C, E, A>
    for EventStore<C, E, A>
{
    async fn load_events(
        &mut self,
        aggregate_id: &str,
        with_metadata: bool,
    ) -> Result<Vec<EventContext<C, E>>, Error> {
        match with_metadata {
            true => {
                self.load_events_with_metadata(aggregate_id)
                    .await
            },
            false => {
                self.load_events_only(aggregate_id)
                    .await
            },
        }
    }

    async fn load_aggregate(
        &mut self,
        aggregate_id: &str,
    ) -> Result<AggregateContext<C, E, A>, Error> {
        match self.with_snapshots {
            true => {
                self.load_aggregate_from_snapshot(aggregate_id)
                    .await
            },
            false => {
                self.load_aggregate_from_events(aggregate_id)
                    .await
            },
        }
    }

    async fn commit(
        &mut self,
        events: Vec<E>,
        context: AggregateContext<C, E, A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventContext<C, E>>, Error> {
        match self.with_snapshots {
            true => {
                self.commit_with_snapshots(events, context, metadata)
                    .await
            },
            false => {
                self.commit_events_only(events, context, metadata)
                    .await
            },
        }
    }
}
