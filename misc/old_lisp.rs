//import/use header
#![feature(box_syntax, box_patterns, slice_patterns)]
#![allow(dead_code)] //TODO: kk removme
#![allow(unused_variables)] //TODO: kk removeme

//#![feature(slice_patterns)]
//replaced with custom char_at #![feature(str_char)]

use std::collections::HashMap;
use std::boxed::Box;
//use std::cell::RefCell;

extern crate list; use list::{Cons, cons_map, car, cdr};
//use list::{cons_len, List};
extern crate err; use err::{debug_p, internal_err};
extern crate types; use types::{Sexps, err};
extern crate lexer; use lexer::lex;
extern crate parser; use parser::parse;

enum FunType {
   BuiltIn(Box<Fn(Box<Cons<Sexps>>) -> Sexps>),
   Lambda(Sexps)
}
struct Callable<'a> { env : SymTable<'a>, f : FunType, arg_names : Cons<String> }
impl<'a> Callable<'a> {
   fn new(arg_names : Cons<String>, f : FunType, /*parent_env : Box<&'a SymTable<'a>>*/) -> Callable<'a> {
      Callable { env: SymTable::new(/*Some(parent_env)*/None), f: f, arg_names: arg_names }
   }
   fn exec(&self, args : Box<Cons<Sexps>>) -> Sexps {
      err("calling .exec of callable");
      match self.f {
         FunType::BuiltIn(ref f) => {
            f(args)
         },
         FunType::Lambda(ref s) => { err("user defined lamda") }
      }
   }
}

//symtable
struct SymTable<'a> {
   bindings : HashMap<String, Callable<'a>>,
   parent : Option<Box<&'a SymTable<'a>>>
}

impl<'a> SymTable<'a> {
   fn new(parent : Option<Box<&'a SymTable<'a>>>) -> SymTable<'a> {
      SymTable {
         bindings : HashMap::new(),
         parent   : parent,
      }
   }
   fn add_defaults(&mut self) {
      let sum_ = |args_ : Box<Cons<Sexps>> | -> Sexps  {
         let mut args = args_;
         let mut sum = 0;
         loop {
            match *args {
               Cons::Cons(Sexps::Num(n), y) => { sum += n; args = y; },
               Cons::Cons(_, _) => { err("bad arguments"); break },
               Cons::Nil   => break,
               _ => return err("bad arguments to sum")
            };
         }
         Sexps::Num(sum)
      };
      let difference_ = |args_ : Box<Cons<Sexps>> | -> Sexps {
         let mut args = args_;
         let mut diff = 0;
         let mut first = true;
         loop {
            match *args {
               Cons::Cons(Sexps::Num(n), y) => {
                  diff = if first { first = false; n } else { diff-n };
                  args = y;
               },
               Cons::Cons(_, _) => { err("bad argument"); break },
               Cons::Nil   => break,
               _ => return err("bad arguments to sum")
            };
         }
         Sexps::Num(diff)
      };

      let sum = Callable::new(Cons::Single("*".to_string()), //* = any arg
                              FunType::BuiltIn(Box::new(sum_))/*,
                              Box::new(self)*/);

      let difference = Callable::new(Cons::Single("*".to_string()),
                                     FunType::BuiltIn(Box::new(difference_))/*,
                                     Box::new(self)*/);
      self.add("+".to_string(), sum);
      self.add("-".to_string(), difference);
      //self.add("-".to_string(), difference)
   }
   fn add(&mut self, key : String, f : Callable<'a>) { self.bindings.insert(key, f); }
   fn lookup(&self, s : &String) -> Option<&Callable> {
      //if !self.bindings.contains_key(s)
      let entry_opt = self.bindings.get(s);
      if let Some(ref entry) = entry_opt { Some(entry.clone()) }
      else {
         if let Some(ref parent) = self.parent { parent.lookup(s) }
         else {
            internal_err("Cannot find symbol in symbol table");
            None //err("None")
         }
      }
   }
}
//end symtable

fn apply_macro(name : &str, args : &Cons<Sexps>, env : &/*mut*/ SymTable) -> Sexps {
   match &name[..] {
      "define" => {
         //match args { Cons::Cons(name, Cons::Cons::(binding, Nil)) }
         //env.add(name.to_string(), );
         err("new define")
      }
      "lambda" => {

         //if let Cons::Cons(x, xs) = *args {}
         //Callable::new(args, env)
         err("new lambda")
      }
      _ => { err("Cannot find symbol in envrionment") }
   }
}

fn eval(exp : &Sexps, env : &mut SymTable) -> Sexps {
   match *exp {
      Sexps::Str(_) | Sexps::Num(_) => exp.clone(), //self evaluation
      Sexps::Sub(_)                 => apply(exp, env),
      Sexps::Err(ref s)             => Sexps::Err(s.clone()),
      Sexps::Var(ref s)             => {
         let lookup_res = env.lookup(&s.clone());
         match lookup_res {
            Some(v)     => v.exec(Box::new(Cons::Nil)),
            None        => err("Undefined variable lookup")
         }
      }
   }
}

fn apply(exp : &Sexps, env : &mut SymTable) -> Sexps {
   match *exp {
      Sexps::Sub(ref e @ box Cons::Cons(_, _)) => {
         err("Calling apply for function");
         let maybe_f = car(e); //get function name
         let maybe_args = cdr(e); //arguments
         if let Some(f) = maybe_f {
            if let Sexps::Var(ref s) = *f { //kk left here
               if let Some(f) = env.lookup(s) {
                  debug_p(2, "Not macro!");
                  if let Some(args) = maybe_args {
                     //kk left here kkleft
                     //f.exec(helper(args))
                     //(cons_map(args, |arg| eval(arg, env)))
                     //f.exec(Box::new(cons_map(&args.clone(), |arg| eval(arg, env))))
                     f.exec(Box::new(cons_map(&args.clone(), |arg| {
                        eval(arg, &mut SymTable::new(Some(Box::new(env))))
                     })))
                  }
                  else { f.exec(Box::new(Cons::Nil)) }
               }
               else { //if can't find symbol assume it's macro
                  debug_p(2, "Macro!");
                  if let Some(args) = maybe_args {
                     apply_macro(s, args, env)
                  } else { err("bad args") }
               }
            }
            else { err("function not var") }
         }
         else { err("bad args") }
      }
      Sexps::Sub(box Cons::Nil) => err("Empty subexpression"),
      _ => err("Bad apply call")
   }
}

fn run(code : &str) -> Sexps {
   let lexemes = lex(code);
   let exp_opt = parse(&lexemes);

   let mut env = SymTable::new(None);
   env.add_defaults();

   if let Some(exp) = exp_opt {
      eval(&exp, &mut env)
   }
   else { err("Couldn't parse code") }
}

fn main() {
   //let code : &str = "(define (6 +) () (+ (test) 5))";
   //let code : &str = "(+ (- 6 4) (+ 3 5))";
   let code : &str = "(+  (+ 1 2) 2 (+ 6 2))"; //-1
   //let code : &str = "(hello (+ world) \"string\")";
   //let code : &str = "(hello (world) (yo test) 5)";
   //let code : &str = "(hello (\"world\"\"test1\" + test) \"another \\\"string\")";

   //lex_test();
   //parse_test();
   //display_sexps(&run(code));
   //print_tree(&run(code), 0);

   interpreter();
}

fn interpreter() {
   use std::io::{self, BufRead};
   let stdin = io::stdin();
   loop {
      let line = stdin.lock().lines().next().unwrap().unwrap();
      let out = run(&line);
      display_sexps(&out)
   }
}

fn helper(a : &list::Cons<Sexps>) -> Box<list::Cons<Sexps>> { Box::new(a.clone()) }

//parse_test()
#[allow(dead_code)]
fn parse_test() {
   let code : &str = "'((6 +) () (+ (test) 5))";
   let lexemes = lex(code);
   let tree_maybe = parse(&lexemes);
   if let Some(tree) = tree_maybe {
      //print_tree(&tree, 0); //println!("{:?}", tree);
   }
   else { internal_err("Parsing failed"); }
}
//lex_test()
#[allow(dead_code)]
fn lex_test() {
   let code : &str = "'(hello (\"world\"\"test1\" + 69 test 42) 47 \"another \\\"string\")";
   let lexemes = lex(code);
   println!("{:?}", lexemes); //print_lexemes(&lexemes);
}


