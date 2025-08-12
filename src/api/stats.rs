use actix_web::{HttpResponse, Responder, get, web};

use super::models::{AppState, StatsResponse};
use crate::blockchain::{DIFF_ADJUST_THRESHOLD_PCT, DIFF_ADJUST_WINDOW, TARGET_BLOCK_TIME_SECS};

#[get("/stats/")]
pub async fn get_stats(state: web::Data<AppState>) -> impl Responder {
    // Snapshot lightweight parts first
    let (height, difficulty, last_interval, avg_interval) = {
        let bc = state.blockchain.lock().expect("mutex poisoned");
        let height = bc.len();
        let difficulty = bc.difficulty();

        // last interval
        let last_interval_secs = if height >= 2 {
            let n = height - 1;
            let newer = &bc.chain[n];
            let older = &bc.chain[n - 1];
            Some((newer.timestamp - older.timestamp).max(0))
        } else {
            None
        };

        // average over the configured window (same logic da blockchain, mas só leitura)
        let avg_secs = if height >= DIFF_ADJUST_WINDOW + 1 {
            let start = height - (DIFF_ADJUST_WINDOW + 1);
            let mut total: i64 = 0;
            for i in (start + 1)..(start + 1 + DIFF_ADJUST_WINDOW) {
                let newer = &bc.chain[i];
                let older = &bc.chain[i - 1];
                total += (newer.timestamp - older.timestamp).max(1);
            }
            Some(total as f64 / DIFF_ADJUST_WINDOW as f64)
        } else {
            None
        };

        (height, difficulty, last_interval_secs, avg_secs)
    };

    // Sizes of mempool and utxo (locks curtos e separados)
    let mempool_size = {
        let mem = state.mempool.lock().expect("mutex poisoned");
        mem.len()
    };
    let utxo_size = {
        let utxo = state.utxo_set.lock().expect("mutex poisoned");
        utxo.len()
    };

    HttpResponse::Ok().json(StatsResponse {
        height,
        difficulty,
        target_block_time_secs: TARGET_BLOCK_TIME_SECS,
        adjust_window: DIFF_ADJUST_WINDOW,
        adjust_threshold_pct: DIFF_ADJUST_THRESHOLD_PCT,
        last_interval_secs: last_interval,
        avg_interval_secs: avg_interval,
        mempool_size,
        utxo_size,
    })
}
