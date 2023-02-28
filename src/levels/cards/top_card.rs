pub async fn gen_top_card(
    users: &[(
        String, //username
        i64,    // rank
        i64,    // level
        i64,    // current xp
    )],
    _guild_name: &str,
) -> anyhow::Result<Vec<u8>> {
    unimplemented!()
}
