extern crate tantivy;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::doc;
use tantivy::Index;
use tempfile::TempDir;
use tantivy::ReloadPolicy;
use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};


#[test]
fn test_full_index() {
    call_full_basic();
}

pub fn call_full_basic() {
    let indexDir = TempDir::new().unwrap();
    println!("{:?}", indexDir);
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT |STORED);

    let schema = schema_builder.build();
    let index = Index::create_in_dir(&indexDir, schema.clone()).unwrap();
    // let tokenizer = tantivy_jieba::JiebaTokenizer {};
    let tokenizer = CangJieTokenizer {
        worker: std::sync::Arc::new(jieba_rs::Jieba::empty()), // empty dictionary
        option: TokenizerOption::Unicode,
    };

    index.tokenizers().register("canjie", tokenizer);

    let mut index_writer = index.writer(50_000_000).unwrap();
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();

    let mut old_man_doc = Document::default();
    old_man_doc.add_text(title, "The Old Man and the Sea");

    old_man_doc.add_text(
        body,
        "He was an old man who fished alone in a skiff in the Gulf Stream and \
         he had gone eighty-four days now without taking a fish.",
    );

    index_writer.add_document(old_man_doc);

    index_writer.add_document(doc!(
        title => "Of Mice and Men",
        body => "A few miles south of Soledad, the Salinas River drops in close to the hillside \
                bank and runs deep and green. The water is warm too, for it has slipped twinkling \
                over the yellow sands in the sunlight before reaching the narrow pool. On one \
                side of the river the golden foothill slopes curve up to the strong and rocky \
                Gabilan Mountains, but on the valley side the water is lined with trees—willows \
                fresh and green with every spring, carrying in their lower leaf junctures the \
                debris of the winter’s flooding; and sycamores with mottled, white, recumbent \
                limbs and branches that arch over the pool"
        ));

    index_writer.add_document(doc!(
        title => "Of Mice and Men",
        body => "A few miles south of Soledad, the Salinas River drops in close to the hillside \
                bank and runs deep and green. The water is warm too, for it has slipped twinkling \
                over the yellow sands in the sunlight before reaching the narrow pool. On one \
                side of the river the golden foothill slopes curve up to the strong and rocky \
                Gabilan Mountains, but on the valley side the water is lined with trees—willows \
                fresh and green with every spring, carrying in their lower leaf junctures the \
                debris of the winter’s flooding; and sycamores with mottled, white, recumbent \
                limbs and branches that arch over the pool"
        ));

    index_writer.add_document(doc!(
            title => "Frankenstein",
            title => "The Modern Prometheus",
            body => "You will rejoice to hear that no disaster has accompanied the commencement of an \
                     enterprise which you have regarded with such evil forebodings.  I arrived here \
                     yesterday, and my first task is to assure my dear sister of my welfare and \
                     increasing confidence in the success of my undertaking."
            ));


    let now = std::time::SystemTime::now();

    for i in 1..1000000 {
        index_writer.add_document(doc!(
                    title => format!("标题{}", i),
                    title => format!("标题{}", i),
                    body => format!("正文正文正文正文正文正文{}", i)
                ));

        // if i % 50000 == 0 {
        //     index_writer.commit().unwrap();
        // }
    }

    println!("index 100w duration is {}ms", now.elapsed().unwrap().as_millis());
    index_writer.commit().unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into().unwrap();


    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![title, body]);
    let query = query_parser.parse_query("").unwrap();
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address).unwrap();
        println!("{}", schema.to_json(&retrieved_doc));
    }
}