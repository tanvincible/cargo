use cargo_test_support::prelude::*;
use cargo_test_support::{project, str};
use std::collections::{HashMap, HashSet};

#[cargo_test]
fn test_non_merging_list_override() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "test-non-merging"
                version = "0.1.0"
            "#,
        )
        .file(
            ".cargo/config.toml",
            r#"
                [registries.custom]
                credential-provider = ["config-file"]
            "#,
        )
        .env("CARGO_REGISTRIES_CUSTOM_CREDENTIAL_PROVIDER", "env-var")
        .build();

    p.cargo("check")
        .with_stderr_data(str![[r#"
[COMPILING] test-non-merging [..]
[FINISHED] [..]
[INFO] Using credential provider: env-var
"#]])
        .run();
}

#[cargo_test]
fn test_merged_config_override() {
    let mut base_config = MergedConfig {
        registries: HashMap::from([(
            "custom".to_string(),
            RegistryCredentialProviderConfig {
                credential_provider: Some(NonMergingList(vec!["config-file".to_string()])),
            },
        )]),
        target: HashMap::from([(
            "x86_64-unknown-linux-gnu".to_string(),
            TargetRunnerConfig {
                runner: Some(NonMergingList(vec!["runner-1".to_string()])),
            },
        )]),
        host: HostRunnerConfig {
            runner: Some(NonMergingList(vec!["host-runner".to_string()])),
        },
        doc: DocBrowserConfig {
            browser: Some(NonMergingList(vec!["firefox".to_string()])),
        },
        credential_alias: HashMap::new(),
        non_merging_fields: HashSet::from([
            "registries.custom.credential-provider".to_string(),
            "target.custom.runner".to_string(),
            "host.runner".to_string(),
            "doc.browser".to_string(),
        ]),
    };

    let override_config = MergedConfig {
        registries: HashMap::from([(
            "custom".to_string(),
            RegistryCredentialProviderConfig {
                credential_provider: Some(NonMergingList(vec!["override-file".to_string()])),
            },
        )]),
        target: HashMap::from([(
            "x86_64-unknown-linux-gnu".to_string(),
            TargetRunnerConfig {
                runner: Some(NonMergingList(vec!["runner-2".to_string()])),
            },
        )]),
        host: HostRunnerConfig {
            runner: Some(NonMergingList(vec!["override-host".to_string()])),
        },
        doc: DocBrowserConfig {
            browser: Some(NonMergingList(vec!["chrome".to_string()])),
        },
        credential_alias: HashMap::from([(
            "alias1".to_string(),
            "value1".to_string(),
        )]),
        non_merging_fields: HashSet::new(),
    };

    let merged_config = base_config.merge(override_config);

    assert_eq!(
        merged_config
            .registries
            .get("custom")
            .unwrap()
            .credential_provider
            .as_ref()
            .unwrap()
            .0,
        vec!["override-file"]
    );

    assert_eq!(
        merged_config
            .target
            .get("x86_64-unknown-linux-gnu")
            .unwrap()
            .runner
            .as_ref()
            .unwrap()
            .0,
        vec!["runner-2"]
    );

    assert_eq!(
        merged_config.host.runner.unwrap().0,
        vec!["override-host"]
    );

    assert_eq!(
        merged_config.doc.browser.unwrap().0,
        vec!["chrome"]
    );

    assert_eq!(
        merged_config.credential_alias.get("alias1").unwrap(),
        "value1"
    );
}
