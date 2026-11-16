use super::*;

const SHAPES: &str = "interface IRunner { void Run(); };
class CBase : public IRunner {
private:
   int m_count;
public:
   virtual void Run() = 0;
   int Count() const { return m_count; }
   void Reset();
};
void CBase::Reset() { m_count = 0; }
class CChild : public CBase {
public:
   void Run() {}
};
";

#[test]
fn mql_members_keep_access_sections_and_pure_virtual_requirements() {
    let result = analyze("shapes.mq5", SHAPES);
    let base = declaration(&result, "CBase");
    assert_eq!(member(base, "m_count")["visibility"], "private");
    assert_eq!(member(base, "Count")["visibility"], "public");
    assert_eq!(member(base, "Run")["required"], true);
    // Reset is declared in the class and defined outside it: one member, public.
    let resets = base["members"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|member| member["name"] == "Reset")
        .count();
    assert_eq!(resets, 1, "{base:#}");
    let child = declaration(&result, "CChild");
    assert_eq!(member(child, "Run")["required"], false);
    assert!(
        member(child, "Count")["origin"]
            .as_str()
            .unwrap()
            .contains("CBase::")
    );
}

#[test]
fn mql_inheritance_resolves_through_included_headers() {
    let root = tempfile::tempdir().unwrap();
    write(&root.path().join("Base.mqh"), "class CBase {};\n");
    write(
        &root.path().join("Nested/Mid.mqh"),
        "#include \"../Base.mqh\"\nclass CMid : public CBase {};\n",
    );
    write(
        &root.path().join("Expert.mq5"),
        "#include <Trade\\Trade.mqh>\n#include \"Nested/Mid.mqh\"\nclass CLeaf : public CMid {};\nCBase *shared;\n",
    );
    let result = report(&run("repo", root.path(), None, Some("structure")));
    let edges = result["structure"]["relationships"].as_array().unwrap();
    let resolved = |source: &str, target: &str, kind: &str| {
        edges.iter().any(|edge| {
            edge["source"]
                .as_str()
                .unwrap()
                .replace('\\', "/")
                .ends_with(source)
                && edge["target"]
                    .as_str()
                    .unwrap()
                    .replace('\\', "/")
                    .ends_with(target)
                && edge["kind"] == kind
                && edge["resolution"] == "resolved"
        })
    };
    assert!(
        resolved(
            "Expert.mq5#source.mql5:CLeaf",
            "Mid.mqh#source.mql5:CMid",
            "inherit"
        ),
        "{edges:#?}"
    );
    assert!(
        resolved(
            "Mid.mqh#source.mql5:CMid",
            "Base.mqh#source.mql5:CBase",
            "inherit"
        ),
        "{edges:#?}"
    );
    assert!(
        resolved("Expert.mq5", "Nested/Mid.mqh", "import"),
        "{edges:#?}"
    );
    // A name only the transitive include provides still resolves.
    assert!(resolved("Expert.mq5", "Base.mqh", "type"), "{edges:#?}");
}

#[test]
fn mql_angle_includes_resolve_in_an_include_folder() {
    let root = tempfile::tempdir().unwrap();
    write(
        &root.path().join("Include/Lib/Util.mqh"),
        "class CUtil {};\n",
    );
    write(
        &root.path().join("Experts/Expert.mq5"),
        "#include <Lib\\Util.mqh>\nclass CApp : public CUtil {};\n",
    );
    let result = report(&run("repo", root.path(), None, Some("structure")));
    let edges = result["structure"]["relationships"].as_array().unwrap();
    assert!(
        edges.iter().any(|edge| edge["kind"] == "inherit"
            && edge["target"].as_str().unwrap().contains("Util.mqh")
            && edge["resolution"] == "resolved"),
        "{edges:#?}"
    );
}

#[test]
fn legacy_mql4_c_subset_has_declarations() {
    let result = analyze(
        "legacy.mq4",
        "struct Order { int ticket; double lots; };\nint start() { return(0); }\n",
    );
    assert_eq!(
        member(declaration(&result, "Order"), "ticket")["visibility"],
        "public"
    );
}
