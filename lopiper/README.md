

===Code in this folder IS WORK IN PROGRESS for LambdaOxide v2. Most of the stuff is broken, so if you want to use the first version of the interpreter, check out parent folder===


Here's small lisp of problems with my previous lisp implementation:
- very rigid Sexps syntax that was awkward to use in Rust, and didn't have binding in LambdaOxide language itself (so you couldn't Lisp expressions in language itself). It also made adding new syntax and build-in functions convoluted and over-complicated.
- cons was not mapped to machine primitive but was instead constructed with lambda functions, which made the performance unacceptable in real-world applications.
- no garbage collection and no tail call optimization, another major omission which made it impractical.
- very awkward implementation of Symbol Table
- very bad error reporting

This new version is a re-write which aims to resolve all the previous problems and omissions, and also serve as a playground for construction of Lisp compiler which translates subset of Scheme (or possible completely different language with lisp syntax) to LLVM bytecode. The plan is to eventually add a JIT to LambdaOxide. An alternative might be to expose compile function to LambdaOxide interpreter, in similar fashion as Common Lisp.

General overview
- Errors: TODO
- Reader (reader.rs) parses one line (or one multi-line expression) to lisp expression. Can return multiple expressions if they're on same line. Create new instance for every line.
- exp.rs has the main Lisp-expression type (Sexps enum)
- Driver (driver.rs) has repl driver and keeps track of reader data (lex/parse error location) and evaluator info (such as stack trace).
- Lexer (lexer.rs) breaks down strings into lisp lexemes
- Parser (parser.rs) converts Lexemes to Sexps
- Compiler (comp.rs) translates primitive language (parsed by Parser) into llvm bytecode
- Evaluator (eval.rs) has the main lisp interpreter (subset of Scheme), not yet finished, but more advanced than the primitive compiled language
- sym_table.rs is symbol table for interpreter
- main.rs parses command line argument and dispatches appropriate action

===Read line===
- every repl input gets it's own stack
- every file read gets it's own stack
- before calling every parse_str for files, set line manually on errors

===Lang syntax===
- "string"
- blah ;semicolon signifies comment until end of line
- test #| multi-line comment block |#

Misc note:
- [ ] Test command line arguments
   - [x] cargo run -- --help
   - [x] cargo run -- --eval (should fail)
   - [x] cargo run -- --eval "blah" (running with eval flag in Eval mode)
   - [x] cargo run -- --eval ha --lex (eval flag in Lex mode)
   - [x] cargo run -- --lex (repl in Lex mode)
   - [x] cargo run -- -f "fds" (file (fds) in Eval mode
   - [ ] cargo run -- -f "fds" --lex --eval "test" (gives error, but should load file and then execute eval)
- [ ] Test parser
   - [ ] cargo run -- --eval "+ 3 5" --lex
- [ ] Test file
   - [ ] cargo run -- --lex -f examples/py_guile.scm
- [ ] Test lexer comments and strings
   - [ ] test incomplete strings and comments
   - [ ] #|test|##|yo|#
   - [ ] "yo";test
   - [ ] (+ "test" blah)
   - [ ] test ;yo
   - [ ] (+ 3 #| hey |#)
   - [ ] strings inside comments, comments inside strings, simple comment inside complex comment
      - [ ] "blah #| yo |#"
      - [ ] #| yo |# "hi"
      - [ ] #| ; |#

Quick TODO:
- [ ] cargo run -- --lex -f examples/py_guile.scm dosen't work because we need parse_line to return Result, run check for result, and if not enough pass more lines to run
- [ ] remove all old code for parse when it returned Sexps instead of Result
- [ ] remove child_parse error and return child's error
- [ ] in get_line of driver, make sure line isn't out of range, and file_lines isn't None
- [ ] implement comment as a type of Lexeme
- [ ] create function from_range_err_to_sexps
- [ ] src/reader parse_line doesn't check return for errors.
- [ ] try running examples/py.lo in guile or drracket
- [ ] try read on old/old-felipe-piper-lisp-idea.lp
- [ ] '() same as Nil
Bugs TODO:
- [ ] repl "-3.3.3" gets lexed as symbol instead of lexer error


Python syntax:
```Python

direct sexps to python translation without special form conversion:
(define (f x y) (+ x y 4 2))
define(f(x, y), +(x, y, 4, 2))

def f(x, y):
   +(x, y, 4 2)
```

TODO:
- [ ] make Scheme translator to python
   - [ ] translate sexps to tabbed python version
   - [ ] generate sexps with python tabs
   - [ ] conditionals
-define and def same thing, def shortcut
- [ ] simple way to load rust libraries, by just passing Environment to their initializer
- [ ] main.rs
   - [ ] use library for command line parsing (https://doc.rust-lang.org/getopts/getopts/index.html)
   - [ ] add option to have --eval and -f. If we have both, then first load file, then eval
   - [ ] if we have eval, then split commands into statements with ; before passing it on to rest of system
   - [ ] possibly move asm_printer, jitter, scm_eval, parse_printer, lex_printer to driver.rs and make them methods of Driver.
- [ ] account for comments in error reporting. Currently comments get deleted from origin.
   - [ ] Possibly add a Comment(Bool, String) (Comment(multiline, comment_data) enum variant to Lexemes
- [ ] JIT/llvm
   - [x] in progress
   - [ ] to Emscripten/Javascript (?)
   - [ ] easily import rust libraries directly, without having to write wrapper for interpreter
- [ ] do something like rustc --explain E0123 with my error codes
- [ ] implement ErrCode::{MisformedFloat, MisformedInt, BadChar}
   - [x] MisFormedNum
- [ ] add single character ''
- [ ] stack trace display
- [x] Make new generic library/crate utils with generic rust stuff.
- [x] check lexer lexer
- [ ] check utils code
- [ ] check parser code
- [ ] don't overoptimize lexer and parser because they don't run often
- [ ] Unit test for every time
- [ ] comments in middle of a line
- [ ] multi-line comments
- [ ] lexer errors for when something begins with a number, but not a number (0sdf or -32fds or 32..12 or 4-3)
- [ ] String::from("blah") or "blah".to_string()?
- [ ] for stack trace, need to implement lambda to work better with define'd names
- [ ] fix quotes for '(+ 3 5) (currently quotes only work for stuff like 'sdf)
- [ ] unit tests
- [ ] performance tests
- [ ] re-write symbol table and make it more robust and faster
- [ ] macros
- [ ] start investigating compiler
- [ ] better error debug output
- [ ] set! set-cell! set-car!
- [ ] possibly for/while/loop so we don't kill the stack
- [ ] support classic scheme define syntax (define (f x) exp exp)
- [ ] let statemnts
- [ ] cond
- [ ] garbage collection
- [ ] low-level fast operators (+ 4 "blah") (+' 3 3) //+' sum prime optimized version
- [ ] stack tracing
- [ ] tail call optimization


/*TODO: copy guile with $3 = last eval result
comma = unquote
//BackQuote = quasiquote
//Q = quote
//(+ 3 5) => 8
//'(+ 3 5) => (+ 3 5)
//(car '(f)) => f
//`((+ 1 2) '(+ 1 2) ,(+ 1 2)) => ((+ 1 2) (quote (+ 1 2)) 3)
//(eval '(+ 3 5)) => 8
//(eval (+ 3 5)) => 8
//(eval ''(+ 3 5)) => '(+ 3 5)
//(eval (eval ''(+ 3 5))) => 8
//'''3 => '''3 or (quote (uqote (quote 3)))
//'fsdf 3 => fsdf 3
*/

(lambda f (a b) (+ (+ 3 5) (+ a b)))
(lambda printf (x))
(lambda main () (print (f 30 4)))

cat output > test.bc
//http://stackoverflow.com/questions/29180737/how-to-generate-executable-from-llvm-ir
llc -march=x86-64 test.bc -o test.s
//as test.s
gcc test.s -o a.out

