use std::convert::TryInto;

use chirp_workflow::prelude::*;
use std::collections::HashMap;

use crate::types;
use crate::ops::get::RouteRow;
use crate::utils::MAX_PATH_COMPONENTS;

#[derive(Debug)]
pub struct Input {
    pub hostname: String,
    pub path: String,
}

#[derive(Debug)]
pub struct Output {
    pub route: Option<types::Route>,
}

/// Generate all path prefixes for a given path
/// For example, "/a/b/c" will generate ["", "/a", "/a/b", "/a/b/c"]
/// Where "" represents the root path
/// Limited to MAX_PATH_COMPONENTS prefixes
fn generate_path_prefixes(path: &str) -> Vec<String> {
    // Always include empty string for root path
    let mut prefixes = vec!["".to_string()];
    
    // If path is empty (root path), just return the empty string
    if path.is_empty() {
        return prefixes;
    }
    
    let mut current = String::new();
    
    // Use take() to limit to MAX_PATH_COMPONENTS but still use a vanilla for loop
    for segment in path.split('/').skip(1).take(MAX_PATH_COMPONENTS) {
        current.push('/');
        current.push_str(segment);
        prefixes.push(current.clone());
    }
    
    prefixes
}

#[operation]
pub async fn get_by_hostname_path(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
    // Get domain_job from configuration
    let domain_job = ctx.config().server()?.rivet.domain_job_for_routes()?;

    // Immediately return None if the hostname doesn't end with domain_job
    // This prevents unnecessary database queries for hostnames that can't possibly match
    if !input.hostname.ends_with(&format!(".{}", domain_job)) {
        return Ok(Output { route: None });
    }
    
    // Normalize path format
    let mut normalized_path = input.path.clone();
    
    // Special case for root path "/"
    if normalized_path == "/" {
        normalized_path = "".to_string();
    } else {
        // Ensure path starts with a slash for non-root paths
        if !normalized_path.starts_with('/') && !normalized_path.is_empty() {
            normalized_path = format!("/{}", normalized_path);
        }
        
        // Remove trailing slash if present
        if normalized_path.ends_with('/') {
            normalized_path.pop();
        }
    }
    
    // Generate all possible path prefixes for subpath routing
    let path_prefixes = generate_path_prefixes(&normalized_path);
    
    // Direct database query to find routes matching hostname and path
    // This query handles both exact matches and subpath routing in a simple union
    let rows = sql_fetch_all!(
        [ctx, RouteRow]
        "
        SELECT 
            route_id,
            namespace_id,
            name_id,
            hostname,
            path,
            route_subpaths,
            selector_tags,
            create_ts,
            update_ts,
            delete_ts,
            100 AS priority -- Exact match gets highest priority
        FROM 
            db_route.routes
        WHERE 
            hostname = $1
            AND path = $2
            AND route_subpaths = false -- Exact matches only
            AND delete_ts IS NULL
            
        UNION ALL
        
        SELECT 
            route_id,
            namespace_id,
            name_id,
            hostname,
            path,
            route_subpaths,
            selector_tags,
            create_ts,
            update_ts,
            delete_ts,
            LENGTH(path) AS priority -- Longer paths get higher priority for subpaths
        FROM 
            db_route.routes
        WHERE 
            hostname = $1
            AND path = ANY($3) -- Use the prefixes we generated in Rust
            AND route_subpaths = true -- Subpath routes only
            AND delete_ts IS NULL
            
        ORDER BY 
            priority DESC, -- Highest priority first
            LENGTH(path) DESC -- Longer paths get precedence within the same priority
        ",
        &input.hostname,
        &normalized_path,
        &path_prefixes
    )
    .await?;

    // Convert the top priority row to a Route, or return None if no matches
    let route = rows
        .into_iter()
        .next() // Get only the top matching route (highest priority due to ORDER BY)
        .map(|row| row.try_into())
        .transpose()?; // Convert Result<Option<T>> to Option<Result<T>>

    Ok(Output { route })
}
