use axum::{ extract::{ Path, State, Query }, Json };
use sqlx::Row as TRow;
use tracing::info;

use crate::{
    server::{ blocks::HashID, blocks::{ TxShort, Paging, Response }, ServerState },
    BlockInfo,
    Error,
};

async fn get_tx_hashes(
    state: &ServerState,
    block: &mut BlockInfo,
    hash: &[u8]
) -> Result<(), Error> {
    let rows = state.db.get_tx_hashes_block(hash).await?;

    let mut tx_hashes: Vec<TxShort> = vec![];
    for row in rows.iter() {
        println!("GET_TX_HASHES_ {:?}", row.columns());
        let hash_id = HashID(row.try_get("hash")?);
        let tx_type: String = row.try_get("tx_type")?;
        tx_hashes.push(TxShort { tx_type, hash_id });
    }

    block.tx_hashes = tx_hashes;

    Ok(())
}

pub async fn get_block_by_hash(
    State(state): State<ServerState>,
    Path(hash): Path<String>
) -> Result<Json<Option<BlockInfo>>, Error> {
    info!("calling /block/hash/:block_hash");

    let id = hex::decode(hash)?;

    let row = state.db.block_by_id(&id).await?;
    let Some(row) = row else {
        return Ok(Json(None));
    };
    let mut block = BlockInfo::try_from(&row)?;

    let block_id: Vec<u8> = row.try_get("block_id")?;
    get_tx_hashes(&state, &mut block, &block_id).await?;

    Ok(Json(Some(block)))
}

pub async fn get_block_by_height(
    State(state): State<ServerState>,
    Path(height): Path<u32>
) -> Result<Json<Option<BlockInfo>>, Error> {
    info!("calling /block/height/:block_height");

    let row = state.db.block_by_height(height).await?;
    let Some(row) = row else {
        return Ok(Json(None));
    };

    let mut block = BlockInfo::try_from(&row)?;

    let block_id: Vec<u8> = row.try_get("block_id")?;
    get_tx_hashes(&state, &mut block, &block_id).await?;

    Ok(Json(Some(block)))
}

pub async fn get_last_block(State(state): State<ServerState>) -> Result<Json<BlockInfo>, Error> {
    let row = state.db.get_last_block().await?;

    let mut block = BlockInfo::try_from(&row)?;

    let block_id: Vec<u8> = row.try_get("block_id")?;
    get_tx_hashes(&state, &mut block, &block_id).await?;

    Ok(Json(block))
}

pub async fn get_blocks(
    State(state): State<ServerState>,
    Query(paging): Query<Paging>
) -> Result<Json<Response>, Error> {
    let rows = state.db.get_blocks(paging.page, paging.page_size).await?;

    let mut blocks = Vec::new();

    for row in rows {
        let mut block = BlockInfo::try_from(&row)?;

        let block_id: Vec<u8> = row.try_get("block_id")?;
        get_tx_hashes(&state, &mut block, &block_id).await?;

        blocks.push(block);
    }

    let total = state.db.get_total_block().await?;

    Ok(Json(Response { data: blocks, total }))
}
