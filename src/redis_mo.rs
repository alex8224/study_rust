use redis::{self, transaction, Commands};

use std::collections::HashMap;

/// This function demonstrates how a return value can be coerced into a
/// hashmap of tuples.  This is particularly useful for responses like
/// CONFIG GET or all most H functions which will return responses in
/// such list of implied tuples.
fn do_print_max_entry_limits(con: &mut redis::Connection) -> redis::RedisResult<()> {
    // since rust cannot know what format we actually want we need to be
    // explicit here and define the type of our response.  In this case
    // String -> int fits all the items we query for.
    let config: HashMap<String, isize> = redis::cmd("CONFIG")
        .arg("GET")
        .arg("*-max-*-entries")
        .query(con)?;

    println!("Max entry limits:");

    println!(
        "  max-intset:        {}",
        config.get("set-max-intset-entries").unwrap_or(&0)
    );
    println!(
        "  hash-max-ziplist:  {}",
        config.get("hash-max-ziplist-entries").unwrap_or(&0)
    );
    println!(
        "  list-max-ziplist:  {}",
        config.get("list-max-ziplist-entries").unwrap_or(&0)
    );
    println!(
        "  zset-max-ziplist:  {}",
        config.get("zset-max-ziplist-entries").unwrap_or(&0)
    );

    Ok(())
}


fn write_java_class(bytes: &[u8]) {
    use std::{fs::File, io::Write};
    use uuid::Uuid;
    let filename = format!("{}.class", Uuid::new_v4().to_string());
    let mut cls_file = File::create(&filename).unwrap();
    cls_file.write(bytes).unwrap();
    println!("write cls file {}", filename);
}

fn do_basic_set_get(con: &mut redis::Connection) -> redis::RedisResult<()> {
   con.set("a", "11")?; 
   let ret_val: String = con.get("a")?;
   println!("get a val is {} , try sadd cmd:", ret_val);
   con.sadd("supcom:name:list", vec!["aaa", "ffff"])?;
   let name_list: Vec<String> = con.smembers("supcom:name:list")?;
   for name in &name_list {
       println!("get name {}", name);
   }

   let all_keys : Vec<String> = con.keys("*")?;
   all_keys.into_iter().for_each(|k| {
       let type_code: String = redis::cmd("type").arg(&k).query(con).unwrap();
       println!("get key === {} type is {}", &k, &type_code);
       if type_code == "string" {
           let ret: Vec<u8> = con.get(&k).unwrap();
           write_java_class(&ret);
           println!("key {} val is ", k);
       }else if type_code == "hash" {
           let map_val: std::collections::HashMap<String, Vec<u8>> = con.hgetall(&k).unwrap();
           map_val.into_iter().for_each(|e| {
               write_java_class( &e.1);
               println!("key {} filed {} ", k, e.0);
           });
       }
   });
   Ok(())
}

/// This is a pretty stupid example that demonstrates how to create a large
/// set through a pipeline and how to iterate over it through implied
/// cursors.
fn do_show_scanning(con: &mut redis::Connection) -> redis::RedisResult<()> {
    // This makes a large pipeline of commands.  Because the pipeline is
    // modified in place we can just ignore the return value upon the end
    // of each iteration.
    let mut pipe = redis::pipe();
    for num in 0..1000 {
        pipe.cmd("SADD").arg("my_set").arg(num).ignore();
    }

    // since we don't care about the return value of the pipeline we can
    // just cast it into the unit type.
    pipe.query(con)?;

    // since rust currently does not track temporaries for us, we need to
    // store it in a local variable.
    let mut cmd = redis::cmd("SSCAN");
    cmd.arg("my_set").cursor_arg(0);

    // as a simple exercise we just sum up the iterator.  Since the fold
    // method carries an initial value we do not need to define the
    // type of the iterator, rust will figure "int" out for us.
    let sum: i32 = cmd.iter::<i32>(con)?.sum();

    println!("The sum of all numbers in the set 0-1000: {}", sum);

    Ok(())
}

/// Demonstrates how to do an atomic increment in a very low level way.
fn do_atomic_increment_lowlevel(con: &mut redis::Connection) -> redis::RedisResult<()> {
    let key = "the_key";
    println!("Run low-level atomic increment:");

    // set the initial value so we have something to test with.
    redis::cmd("SET").arg(key).arg(42).query(con)?;

    loop {
        // we need to start watching the key we care about, so that our
        // exec fails if the key changes.
        redis::cmd("WATCH").arg(key).query(con)?;

        // load the old value, so we know what to increment.
        let val: isize = redis::cmd("GET").arg(key).query(con)?;

        // at this point we can go into an atomic pipe (a multi block)
        // and set up the keys.
        let response: Option<(isize,)> = redis::pipe()
            .atomic()
            .cmd("SET")
            .arg(key)
            .arg(val + 1)
            .ignore()
            .cmd("GET")
            .arg(key)
            .query(con)?;

        match response {
            None => {
                continue;
            }
            Some(response) => {
                let (new_val,) = response;
                println!("  New value: {}", new_val);
                break;
            }
        }
    }

    Ok(())
}

/// Demonstrates how to do an atomic increment with transaction support.
fn do_atomic_increment(con: &mut redis::Connection) -> redis::RedisResult<()> {
    let key = "the_key";
    println!("Run high-level atomic increment:");

    // set the initial value so we have something to test with.
    con.set(key, 42)?;

    // run the transaction block.
    let (new_val,): (isize,) = transaction(con, &[key], |con, pipe| {
        // load the old value, so we know what to increment.
        let val: isize = con.get(key)?;
        // increment
        pipe.set(key, val + 1).ignore().get(key).query(con)
    })?;

    // and print the result
    println!("New value: {}", new_val);

    Ok(())
}


/// Runs all the examples and propagates errors up.
pub fn do_redis_code(url: &str) -> redis::RedisResult<()> {
    // general connection handling
    let client = redis::Client::open(url)?;
    let mut con = client.get_connection()?;

    do_basic_set_get(&mut con)?;
    // read some config and print it.
    do_print_max_entry_limits(&mut con)?;

    // demonstrate how scanning works.
    do_show_scanning(&mut con)?;

    // shows an atomic increment.
    do_atomic_increment_lowlevel(&mut con)?;
    do_atomic_increment(&mut con)?;

    Ok(())
}


