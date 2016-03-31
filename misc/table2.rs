#![allow(dead_code, unused_variables)]
#![feature(box_syntax, box_patterns)]
#![feature(as_unsafe_cell)]

//#![allow(unused_variables)]
//#![allow(unused_imports)]
use std::collections::HashMap;
use std::boxed::Box;

use std::rc::Rc;
use std::cell::RefCell;

//extern crate list; use list::{Cons, cons, cons_reverse, car, cdr};
//extern crate utils; use utils::{print_space, print_nest, char_at, is_numeric};

extern crate types; use types::{Sexps, err, display_sexps, print_tree, EnvId};

//tmp TODO kkleft: move main interpreter to separate file
//and only use this one for tables
extern crate err; use err::debug_p;
extern crate lexer; use lexer::lex;
extern crate parser; use parser::parse;
extern crate list; use list::{Cons, car, cdr, cons_map};
//end tmp

type FunArgNames = Sexps; //Option<Vec<String>>;
type FunArgs = Sexps;
//type EnvId = Option<u32>; type EnvId = u32;


type Root<'a> = &'a RefCell<Env>;

//callable
enum Callable {
   BuiltIn(EnvId, Box<Fn(Sexps, Root, EnvId) -> Sexps>), //args, root, our env
   Lambda(EnvId, FunArgNames, Sexps)
}
impl Callable {
   fn exec(&self, args_exp : Sexps, root : Root) -> Sexps {
      err("calling .exec of callable");

      match *self {
         Callable::BuiltIn(parent, ref f) => { f(args_exp, root, parent) },
         Callable::Lambda(ref t, ref arg_names_exp, ref exp) => {
            debug_p(2, "Calling .exec() of lambda");
            if let Sexps::Sub(box ref arg_names_@Cons::Cons(Sexps::Var(_), _)) = *arg_names_exp {
               //if let Sexps::Sub(box = ) arg_names
               let mut arg_names : &Cons<Sexps> = arg_names_;
               while let Cons::Cons(Sexps::Var(ref arg_name), box ref rest) = *arg_names {
                  root.borrow_mut().table_add(t.clone(), &*arg_name, make_sym_table_val(Sexps::Num(5)));
                  arg_names = &rest;
               }
            };
            /*if let Sexps::Sub(box args_@Cons::Cons(Sexps::Var(_), _) = args_exp {
               //if let Sexps::Sub(box = ) arg_names
               let mut args : Cons<Sexps> = args_;
               while let Cons::Cons(Sexps::Var(arg), box rest) {
                  args = rest;
               }
            }*/

            //let new_table = root.borrow_mut().table_new(parent.clone());
            //if let Sexps::Sub(box args_@Cons::Cons(Sexps::Var(arg), box rest) = args_ {
            //root.borrow_mut().table_add(new_table, "~", make_sym_table_val(args));
            //let mut env = SymTable::new(root, env_parent.clone());
            /*if let Some(ref arg_names) = *arg_names_opt {
               //let mut i = 0;
               //for arg in args { env.add(arg_names[i], arg); i+=1; }
            } else { kkleft: env.add("*", args); } */
            apply(exp, root, t.clone())
         }
      }
   }
}

fn make_sym_table_val(exp : Sexps) -> Callable {
   //let root = Env::new();
   let ret : Box<Fn(Sexps, Root, EnvId) -> Sexps> = Box::new(move |args, root, env| -> Sexps {
      exp.clone()
   });
   Callable::BuiltIn(0, ret)
}
fn get_sym_table_val(v : Option<&Callable>) -> Sexps {
   match v {
      None => err("Not found"),
      Some(f) => f.exec(err(""), &(RefCell::new(Env::new())))
   }
}
//end callable

struct Table { bindings: HashMap<String, Callable>, parent: EnvId }
struct Env { tables : Vec<Table>, }

impl Env {
   fn new() -> Env {
      let mut env = Env { tables : Vec::new() };
      env.table_new(0);
      env
   }
   fn table_new(&mut self, parent : EnvId) -> EnvId {
      self.tables.push(Table { bindings : HashMap::new(), parent: parent });
      self.tables.len()-1
   }
   fn table_add(&mut self, table_id : EnvId, key : &str, entry : Callable) -> bool {
      let mut i = 0;
      for table in self.tables.iter_mut() {
         if i == table_id {
            table.bindings.insert(key.to_string(), entry);
            return true;
         }
         i += 1;
      }
      false
   }
   fn lookup(&self, table_id : EnvId, key : &str) -> Option<&Callable> {
      match self.tables.get(table_id) {
         Some(table) => {
            let entry_opt = table.bindings.get(key);
            match entry_opt {
               Some(val) => return Some(val),
               None if table_id == 0 => return None,
               None => self.lookup(table.parent, key)
            }
         }
         None => return None
      }
   }
   fn add_defaults(&mut self) {
      let sum_ = |args_sexps : Sexps, root : Root, table : EnvId| -> Sexps {
         let mut sum = 0;
         if let Sexps::Sub(box ref args_) = args_sexps {
            let mut args : &Cons<Sexps> = args_; //Box::new(args_);

            loop {
               match *args {
                  Cons::Cons(Sexps::Num(n), ref ns) => { sum += n; args = ns; },
                  Cons::Cons(_, _) => { err("bad arguments"); break },
                  Cons::Nil   => break,
                  _ => return err("bad arguments to sum")
               };
            }
            Sexps::Num(sum)
         }
         else { err("bad arguments") }
      };

      let  sum = Callable::BuiltIn(0, Box::new(sum_));
      self.table_add(0, "+", sum);

      let diff_ = |args_sexps : Sexps, root : Root, table : EnvId| -> Sexps {
         let mut diff = 0;
         let mut first = true;

         if let Sexps::Sub(box ref args_) = args_sexps {
            let mut args : &Cons<Sexps> = args_; //Box::new(args_);

            loop {
               match *args {
                  Cons::Cons(Sexps::Num(n), ref ns) => {
                     if first { diff = n; first = false; } else { diff -= n; }
                     args = ns;
                  },
                  Cons::Cons(_, _) => { err("bad arguments"); break },
                  Cons::Nil   => break,
                  _ => return err("bad arguments to sum")
               };
            }
            Sexps::Num(diff)
         }
         else { err("bad arguments") }
      };

      let diff = Callable::BuiltIn(0, Box::new(diff_));
      self.table_add(0, "-", diff);

   }
}

fn eval(exp : &Sexps, root : Root, table : EnvId) -> Sexps {
   //root.borrow_mut().table_add(0, "hello", make_sym_table_val(err("test")));
   //print_tree(&get_sym_table_val(root.borrow_mut().lookup(0, "hello")), 0);
   //root.borrow_mut().table_add(0, "hello", make_sym_table_val(Sexps::Num(5)));
   println!("evalualting: ");
   print_tree(exp, 2);

   match *exp {
      Sexps::Str(_) | Sexps::Num(_) => exp.clone(), //self evaluation
      Sexps::Sub(_)                 => apply(exp, root, table),
      //Sexps::Lambda(..)           => apply(exp, root, table),
      ref l@Sexps::Lambda(..)       => l.clone(),
      Sexps::Err(ref s)             => Sexps::Err(s.clone()),
      Sexps::Var(ref s)             => {
         let borrowed = root.borrow();
         let lookup_opt = borrowed.lookup(table, &s.clone());
         //comment start
         let x = get_sym_table_val(lookup_opt);
         println!("eval variable {}", s);
         print_tree(&x, 0);
         x
         //comment end
         //get_sym_table_val(lookup_opt)
      }
   }
}

fn apply_macro(name : &str, args : &Cons<Sexps>, root : Root, t : EnvId) -> Sexps {
   match &name[..] {
      "define" => {
         err("new define");
         if let Cons::Cons(Sexps::Var(ref name), ref binding) = *args {
            //let eval_result = eval(&Sexps::Sub(binding.clone()), root, t);
            //print_tree(&Sexps::Sub(binding.clone()), 0);
            let eval_result = if let Some(x) = car(binding) {
               println!("defining: {}", &*name);
               print_tree(x, 2);
               make_sym_table_val(eval(x, root, t))
            }
            else { make_sym_table_val(err("bad define")) };
            root.borrow_mut().table_add(t, name, eval_result);
            //print_tree(&get_sym_table_val(Some(&eval_result)), 0);
            Sexps::Var("success".to_string())
         }
         else { err("bad define syntax") }
      }
      "lambda" => {
         if let Cons::Cons(ref args@Sexps::Sub(_), box Cons::Cons(ref exp, _)) = *args {
            let borrowed = unsafe { root.as_unsafe_cell().get() };

            println!("args and exp:");
            print_tree(args, 2); print_tree(exp, 4);

            let lambda = Callable::Lambda(t, args.clone(), exp.clone());
            let lambda_table = unsafe { (*borrowed).table_new(t) };
            root.borrow_mut().table_add(lambda_table, "self", lambda);
            Sexps::Lambda(lambda_table, "self".to_string())
            //var params = kkzz
            //if let Cons::Cons(x, xs) = *args {}
            //Callable::new(args, env)
         } else { err("bad arguments to lambda") }
      }
      _ => { err(&*format!("Cannot find symbol in envrionment and not macro {}", name.to_string())) }
   }
}

fn apply(exp : &Sexps, root : Root, table : EnvId) -> Sexps {
   match *exp {
      Sexps::Sub(/*ref e @*/ box Cons::Cons(ref f, ref args)) => {
         debug_p(2, "Calling apply for function");

         if let Sexps::Var(ref s) = *f { //if first element is variable look it up
            //let func_lookup = root.borrow().lookup(table, s);
            debug_p(2, &format!("applying: {}", s));
            let borrowed = unsafe { root.as_unsafe_cell().get() };
            let func_lookup = unsafe { (*borrowed).lookup(table, s) };

            if let Some(f) = func_lookup { //if function look up successful
               debug_p(2, "Found symbol, executing function");
               let evaled_args = cons_map(&args.clone(), |arg| {
                  //let new_env = borrowed.table_new(table);
                  let new_table = unsafe { (*borrowed).table_new(table) };
                  eval(arg, root, new_table)
                  //eval(arg, root, root.borrow().table_new(table))
               });
               let result = f.exec(Sexps::Sub(Box::new(evaled_args)), root);
               /*if let Sexps::Lambda(ref table, ref name) = result {
                  if let Some(call) = unsafe { (*borrowed).lookup(*table, name) } {
                     call.exec(Sexps::Sub(Box::new(Cons::Nil)), root) //Box::new(evaled_args)
                  } else { err("bad lambda") }
               }
               else { result }*/
               result
            }
            else { //if can't find symbol assume it's macro
               debug_p(2, "Macro!");
               apply_macro(s, &args, root, table)
            }
         }
         /*else if let Sexps::Lambda(ref table, ref name) = *f { //TODO kkleft: removeme
            debug_p(2, "applying lambda");
            if let Some(lambda) = root.borrow().lookup(*table, &*name) {
               lambda.exec(Sexps::Sub(args.clone()), root)
            } else { err("lambda not found") }
         }*/
         else { err("(x y z) <- x has to be macro or function") }
         /*let maybe_f = car(e); //get function name
         let maybe_args = cdr(e); //arguments

         if let Some(f) = maybe_f { //first element of sexps
            if let Sexps::Var(ref s) = *f { //if first element is variable look it up
               if let Some(f) = root.lookup(table, s) { //if look up successful
                  debug_p(2, "Not macro!");
                  if let Some(args) = maybe_args { //if we have rest of arguments
                     //f.exec(helper(args))
                     //(cons_map(args, |arg| eval(arg, env)))
                     //f.exec(Box::new(cons_map(&args.clone(), |arg| eval(arg, env))))
                     f.exec(Box::new(cons_map(&args.clone(), |arg| {
                        //eval(arg, &mut SymTable::new(Some(Box::new(env))))
                        eval(arg, root, root.table_new(table))
                     })))
                  }
                  else { f.exec(Box::new(Cons::Nil)) }
               }
               else { //if can't find symbol assume it's macro
                  debug_p(2, "Macro!");
                  if let Some(args) = maybe_args {
                     apply_macro(s, args, root, table)
                  } else { err("bad args") }
               }
            }
            else { err("function not var") }
         }
         else { err("bad args") } */
    }
      Sexps::Sub(box Cons::Nil) => err("Empty subexpression"),
      _ => err("Bad apply call")
   }
}

fn run(root : Root, code : &str) -> Sexps {
   let lexemes = lex(code);
   let exp_opt = parse(&lexemes);

   if let Some(exp) = exp_opt {
      eval(&exp, root, 0) //was &mut root
   }
   else { err("Couldn't parse code") }
}

fn interpreter() {
   use std::io::{self, BufRead};
   let stdin = io::stdin();

   let mut root = RefCell::new(Env::new());
   root.borrow_mut().add_defaults();

   let mut cmd;
   cmd = "(define f (lambda (x) (+ x x)))";
   println!("> {}", cmd);
   display_sexps(&run(&root, cmd));
   cmd = "(f 5)";
   println!("> {}", cmd);
   display_sexps(&run(&root, "(f 5)"));

   loop {
      print!(">");
      use std::io::{self, Write};
      io::stdout().flush().unwrap();
      let line = stdin.lock().lines().next().unwrap().unwrap();
      let out = run(&root, &line);
      display_sexps(&out)
   }
}


fn main() {
   interpreter();
}

fn table_test() {
   let mut x : Env = Env::new();
   let child = x.table_new(0);
   let child2 = x.table_new(child);

   x.table_add(child, "hello", make_sym_table_val(err("test")));
   x.table_add(child2, "test", make_sym_table_val(err("yo")));
   x.table_add(child, "hello3", make_sym_table_val(err("yo2")));

   display_sexps(&get_sym_table_val(x.lookup(child2, "hello")));

}


