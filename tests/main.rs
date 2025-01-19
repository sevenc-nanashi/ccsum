#[rstest::rstest]
#[test]
#[case(&[])]
#[case(&["--tag"])]
fn test_match(#[case] args: &[&str]) -> anyhow::Result<()> {
    let files = glob::glob(concat!(env!("CARGO_MANIFEST_DIR"), "/src/*.rs"))?
        .collect::<Result<Vec<_>, _>>()?;

    let ccsum_out = assert_cmd::Command::cargo_bin("ccsum")?
        .args(["-a", "sha256"])
        .args(args)
        .args(&files)
        .unwrap();
    let sha256_out = assert_cmd::Command::new("sha256sum")
        .args(args)
        .args(&files)
        .unwrap();

    assert_eq!(
        std::str::from_utf8(&ccsum_out.stdout)?,
        std::str::from_utf8(&sha256_out.stdout)?
    );

    Ok(())
}

#[test]
fn test_check() -> anyhow::Result<()> {
    let files = glob::glob(concat!(env!("CARGO_MANIFEST_DIR"), "/src/*.rs"))?
        .collect::<Result<Vec<_>, _>>()?;

    let sha256_out = assert_cmd::Command::new("sha256sum").args(&files).unwrap();
    let sha256_out = std::str::from_utf8(&sha256_out.stdout)?;

    let ccsum_out = assert_cmd::Command::cargo_bin("ccsum")?
        .args(["-a", "sha256"])
        .args(&files)
        .unwrap();
    let ccsum_out = std::str::from_utf8(&ccsum_out.stdout)?;

    assert!(assert_cmd::Command::cargo_bin("ccsum")?
        .args(["-a", "sha256", "-c"])
        .write_stdin(ccsum_out.as_bytes())
        .unwrap()
        .status
        .success());
    assert!(assert_cmd::Command::cargo_bin("ccsum")?
        .args(["-a", "sha256", "-c"])
        .write_stdin(sha256_out.as_bytes())
        .unwrap()
        .status
        .success());
    assert!(assert_cmd::Command::new("sha256sum")
        .args(["-c"])
        .write_stdin(ccsum_out.as_bytes())
        .unwrap()
        .status
        .success());

    Ok(())
}
