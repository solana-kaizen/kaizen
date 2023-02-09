#[cfg(test)]
mod tests {
    use crate::identity::program::*;
    use kaizen::emulator::Simulator;
    use kaizen::prelude::*;
    use kaizen::result::Result;

    #[allow(unused_imports)]
    use std::str::FromStr;

    #[async_std::test]
    async fn identity_init() -> Result<()> {
        kaizen::container::registry::init()?;

        let program_id = generate_random_pubkey();
        let simulator = Simulator::try_new_for_testing()?
            .with_mock_accounts(program_id, None)
            .await?;

        let config = InstructionBuilderConfig::new(simulator.program_id())
            .with_authority(&simulator.authority())
            .with_sequence(0u64);

        let builder = InstructionBuilder::new_with_config_for_testing(&config)
            .with_generic_account_templates_with_seeds(&[(AddressDomain::Authority, b"proxy")])
            .with_generic_account_templates(1)
            .seal()?;

        let accounts = builder.generic_template_accounts();
        let proxy = accounts[0].clone(); // PDA0
        let identity = accounts[1].clone();

        simulator
            .execute_handler(builder, |ctx: &ContextReference| {
                log_trace!("create identity");
                Identity::create(ctx)?;
                Ok(())
            })
            .await?;

        let proxy_pubkey =
            find_identity_proxy_pubkey(&simulator.program_id(), &simulator.authority())?;
        log_trace!(
            "validating proxy pubkey: {} vs {}",
            proxy.pubkey,
            proxy_pubkey
        );
        assert_eq!(proxy.pubkey, proxy_pubkey);

        let config = config.with_identity(&identity.pubkey);

        // load test container
        let builder = InstructionBuilder::new_with_config_for_testing(&config).seal()?;

        simulator
            .execute_handler(builder, |ctx: &ContextReference| {
                log_trace!("testing authority presense in the identity");
                let identity = ctx.try_identity()?;
                assert!(identity.try_has_authority(ctx.authority.key)?);
                Ok(())
            })
            .await?;

        Ok(())
    }
}
