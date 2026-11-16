mod support;
use support::function;

const SOURCE: &str = r#"#property strict
input int Period = 14;
sinput color Tint = clrRed;

class Signal {
public:
   int Score(int value) {
      if (value > 0 && value < 10) return 1;
      else if (value == 0) return 0;
      return -1;
   }
};

int OnInit() {
   for (int i = 0; i < 10; i++) {
      switch (i) { case 1: break; case 2: break; default: break; }
   }
   return INIT_SUCCEEDED;
}
"#;

#[test]
fn mql4_mql5_and_header_files_share_metrics() {
    let temp = tempfile::tempdir().unwrap();
    for extension in ["mq4", "mq5", "mqh"] {
        let path = temp.path().join(format!("expert.{extension}"));
        support::write(&path, SOURCE);
        let report = support::report(&support::run("file", &path, None, None));
        let file = &report["files"][0];
        assert_eq!(file["language"], "mql5", "{extension}");
        // The two cases add paths; `default` does not.
        assert_eq!(function(file, "OnInit")["cyclomatic_complexity"], 4);
        assert_eq!(function(file, "Score")["cyclomatic_complexity"], 4);
        assert_eq!(function(file, "Score")["cognitive_complexity"], 3);
    }
}

#[test]
fn mql_and_cpp_files_in_one_repository_keep_their_languages() {
    let temp = tempfile::tempdir().unwrap();
    support::write(&temp.path().join("expert.mq5"), SOURCE);
    support::write(
        &temp.path().join("engine.cpp"),
        "int run(int x) { return x ? 1 : 0; }\n",
    );
    let report = support::report(&support::run("repo", temp.path(), None, None));
    let language = |suffix: &str| support::file(&report, suffix)["language"].clone();
    assert_eq!(language("expert.mq5"), "mql5");
    assert_eq!(language("engine.cpp"), "cpp");
}
