#![allow(dead_code)]

use oxicloak::*;
use err::DEBUG;
use std::boxed::Box;
use list::{Cons, cons_map, cons};
use utils::print_nest;
use self::Sexps::*;
use main::{Env, Callable};
use std::cell::RefCell;

pub type EnvId = usize;

pub type FunArgNames = Sexps;
pub type FunArgs = Sexps;

pub type Root<'a> = &'a RefCell<Env>;
pub type BuiltInFunc = Fn(Sexps, Root, EnvId) -> Sexps;

#[derive(Debug, PartialEq, Clone)]
pub enum QuoteType {
   BackQuote, Comma, Q
}

#[derive(Debug, PartialEq)]
pub enum Lexeme {
   OpenParen, CloseParen, Str(String), Sym(String), Int(i64), Float(f64), Quote(QuoteType)
}

#[derive(Clone, Debug)] //Try to implement copy
pub enum Sexps {
   Str(String), Int(i64), Float(f64), Err(String), Bool(bool), Var(String),
   //(envid and name?)
   Lambda(EnvId, String), Quote(QuoteType), Sub(Box<Cons<Sexps>>),
   //Sub(Box<Vec<Sexps>>), Literal(String) //literal is var now
}

impl PartialEq for Sexps {
   fn eq(&self, other: &Sexps) -> bool {
      match (self, other) {
         (&Str(ref s1), &Str(ref s2))     => s1 == s2,
         (&Int(ref n1), &Int(ref n2))     => n1 == n2,
         (&Float(ref n1), &Float(ref n2)) => n1 == n2,
         (&Float(ref n1), &Int(ref n2))   => n1.clone() == (n2.clone() as f64),
         (&Int(ref n1), &Float(ref n2))   => (n1.clone() as f64) == n2.clone(),
         (&Bool(ref b1), &Bool(ref b2))   => b1 == b2,
         _                                => false
      }
   }
}

use std::cmp::Ordering;
impl PartialOrd for Sexps {
   fn partial_cmp(&self, other: &Sexps) -> Option<Ordering> {
      use std::cmp::Ordering::*;
      match (self, other) {
         (&Int(ref n1), &Int(ref n2)) if n1 < n2  => Some(Less),
         (&Int(ref n1), &Int(ref n2)) if n1 > n2  => Some(Greater),
         //(&Int(ref n1), &Int(ref n2)) if n1 == n2 => Some(Equal),
         (&Float(ref n1), &Float(ref n2)) if n1 > n2  => Some(Greater),
         (&Float(ref n1), &Float(ref n2)) if n1 < n2  => Some(Less),

         (&Float(ref n1), &Int(ref n2)) if n1.clone() > (n2.clone() as f64)
            => Some(Greater),
         (&Float(ref n1), &Int(ref n2)) if n1.clone() < (n2.clone() as f64)
             => Some(Less),

         (&Int(ref n1), &Float(ref n2)) if (n1.clone() as f64) > n2.clone()
            => Some(Greater),
         (&Int(ref n1), &Float(ref n2)) if (n1.clone() as f64) < n2.clone()
             => Some(Less),

         _                                => None
      }
   }
}

impl Drop for Sexps {
   fn drop(&mut self) {
      match *self {
         Sexps::Err(ref s) if DEBUG >= 5 => println!("err dropping: {}", s),
         _ if DEBUG >= 7 => println!("sexps going out of scope: {:?}", self),
         _ => {}
      }
   }
}

//results, Lex fail, parse fail, eval fails
#[derive(Debug)]
pub enum LexFail {}
#[derive(Debug)]
pub enum EvalFail {}
#[derive(Debug)]
pub enum ParseFail {
   NoStartParen, NoEndParen, ExtraCloseParen, ChildParseFail, BadLexeme
}
pub type ParseResult = Result<Sexps, (ParseFail, usize)>;

#[derive(Debug)]
pub enum RunFail {
   FailParse(ParseFail), FailLex(LexFail), FailEval(EvalFail), UncompleteExp,
}
pub type RunResult = Result<Sexps, (RunFail, usize)>;

pub fn display_run_result(res : &RunResult) {
   match *res {
      Ok(ref exp) => display_sexps(exp),
      _           => println!("error: {:?}", res)
   }
}
//end result, and failure enums

pub fn char_to_quote(c : char) -> Option<QuoteType> {
   match c {
      '`'   => Some(QuoteType::BackQuote),
      '\''  => Some(QuoteType::Q),
      ','   => Some(QuoteType::Comma),
      _     => None
   }
}
pub fn quote_to_str(q : QuoteType) -> String {
   match q {
      QuoteType::BackQuote   => "`",
      QuoteType::Q           => "'",
      QuoteType::Comma       => ","
   }.to_string()
}

pub fn same_type(exp1 : &Sexps, exp2 : &Sexps) -> bool {
   let mut same = false;
   match *exp1 {
      Sexps::Str(..)     => if let Sexps::Str(..) = *exp2 { same = true; },
      Sexps::Int(..)     => if let Sexps::Int(..) = *exp2 { same = true; },
      Sexps::Float(..)   => if let Sexps::Float(..) = *exp2 { same = true; },
      Sexps::Var(..)     => if let Sexps::Var(..) = *exp2 { same = true; },
      Sexps::Err(..)     => if let Sexps::Err(..) = *exp2 { same = true; },
      Sexps::Sub(..)     => if let Sexps::Sub(..) = *exp2 { same = true; },
      Sexps::Lambda(..)  => if let Sexps::Lambda(..) = *exp2 { same = true; },
      Sexps::Bool(..)    => if let Sexps::Bool(..) = *exp2 { same = true; },
      Sexps::Quote(..)   => if let Sexps::Quote(..) = *exp2 { same = true; }
   }
   same
}
pub fn arg_extractor(exp : &Sexps) -> Option<Vec<Sexps>> {
   let mut ret = Vec::new();

   if let Sexps::Sub(box ref args_) = *exp {
      let mut args =  args_;
      loop {
         if let Cons::Cons(ref arg, ref rest) = *args {
            if let Sexps::Sub(_) = *arg { return None; }
            else {
               ret.push(arg.clone());
               args = rest;
            }
         } else { break; }

      }
      Some(ret)
   }
   else { None }
}

pub fn arg_extract_str(args : &Vec<Sexps>, index : usize) -> Option<String> {
   if let Sexps::Str(ref s) = args[index] {
      Some(s.clone())
   } else { None }
}
pub fn arg_extract_num(args : &Vec<Sexps>, index : usize) -> Option<f64> {
   if let Sexps::Float(ref s) = args[index] {
      Some(s.clone())
   } else if let Sexps::Int(ref s) = args[index] {
      Some(s.clone() as f64)
   } else { None }
}
pub fn arg_extract_int(args : &Vec<Sexps>, index : usize) -> Option<i64> {
    if let Sexps::Int(ref s) = args[index] { Some(s.clone()) } else { None }
}
pub fn arg_extract_float(args : &Vec<Sexps>, index : usize) -> Option<f64> {
   if let Sexps::Float(ref s) = args[index] { Some(s.clone()) } else { None }
}
pub fn arg_extract_bool(args : &Vec<Sexps>, index : usize) -> Option<bool> {
   if let Sexps::Bool(ref b) = args[index] { Some(b.clone()) } else { None }
}

#[allow(unused_variables)]
pub fn make_sym_table_val(exp : Sexps) -> Callable {
   //let root = Env::new();
   let ret : Box<Fn(Sexps, Root, EnvId) -> Sexps> = Box::new(move |args, root, env| -> Sexps {
      cons_to_sexps(cons(err("__var"), cons(exp.clone(), Cons::Nil)))
   });
   Callable::BuiltIn(0, ret)
}
pub fn sym_table_is_var(v : Option<&Callable>) -> bool {
   if let Some(f) = v {
      match f.exec(err("__sym"), &(RefCell::new(Env::new()))) {
         Sub(box Cons::Cons(Err(ref s), _)) if s == "__var" => true,
         _ => false
      }
   } else { false }
}
pub fn get_sym_table_val(v : Option<&Callable>) -> Sexps {
   if let Some(f) = v {
      match f.exec(err("__sym"), &(RefCell::new(Env::new()))) {
         Sexps::Sub(box Cons::Cons(Sexps::Err(ref s), box Cons::Cons(ref exp, _))) if s == "__var"
            => exp.clone(),
         _ => err("Bad value")
      }
   } else { err("Not found") }
}

//works well, but we have derive(Debug) on lexemes so we can just debug print them
pub fn print_lexemes(lexemes: &Vec<Lexeme>) {
   for l in lexemes.iter() {
      match *l {
         /*_ => {} empty match */
         Lexeme::OpenParen    => println!("open paren"),
         Lexeme::CloseParen   => println!("close paren"),
         Lexeme::Str(ref s)   => println!("string {}", s),
         Lexeme::Sym(ref s)   => println!("sym {}", s),
         Lexeme::Int(ref n)   => println!("integer {}", n),
         Lexeme::Float(ref n) => println!("float {}", n),
         Lexeme::Quote(ref q) => println!("quote: {}", quote_to_str(q.clone()))
      }
   }
}
pub fn display_sexps(exp: &Sexps) {
   match *exp {
      Str(ref s)  => println!("{}", s),
      Int(ref n)  => println!("{}", n),
      Float(ref n)=> println!("{}", n),
      Var(ref s)  => println!("{}", s),
      Err(ref s)  => println!("{}", s),
      Lambda(..)  => println!("<lambda>"),
      Bool(x)     => println!("{}", x),
      Sub(..)     => print_compact_tree(exp),
      Quote(ref q)=> println!("{}", quote_to_str(q.clone()))
      //_                 => println!("bad sexps, cant print")
   }
}
fn print_compact_tree_helper(t: &Sexps) {
   match *t {
      Sub(box ref sub) => { //box ref sexps
         print!("(");
         //kk for x in sub { print_tree(&x, deepness+4); }
         cons_map(sub, |x| print_compact_tree_helper(x));
         print!(")");
      },
      _ => { print!("{:?} ", t) }
   }
}
pub fn print_compact_tree(t: &Sexps) {
   print_compact_tree_helper(t);
   println!("");
}
pub fn print_tree(t: &Sexps, deepness: u8) {
   match *t {
      Sub(box ref sub) => { //box ref sexps
         print_nest("(", deepness, None);
         //kk for x in sub { print_tree(&x, deepness+4); }
         cons_map(sub, |x| print_tree(x, deepness+4));
         print_nest(")", deepness, None);
      },
      _ => { print_spaces(deepness); println!("{:?}", t) }
   }
}
pub fn cons_to_sexps(c : Cons<Sexps>) -> Sexps { Sub(Box::new(c)) }
pub fn err(s : &str) -> Sexps { Err(s.to_string()) } //or String::from(s)
