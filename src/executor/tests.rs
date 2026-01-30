use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use super::{
    runtime::LogRuntime,
    getter::IoGetter,
    LineByLineExecuter,
};

use crate::{ContextIoGetter, ContextRuntime, build_rules, executor::appliable_rules::build_rules_with_context, io_macros::{await_io, io_fn, io_test}, tests::{NoGet, NoLog}};

struct SingleGetter(&'static str);

impl IoGetter for SingleGetter {
    #[io_fn(impl)]
    fn get_io(&mut self, _: &str) -> Result<String, String> {
        Ok(self.0.to_string())
    }
}

#[io_test(pollster::block_on)]
fn line_by_line_getter() {
    let get_b = SingleGetter("b");

    let rules = "GET var :\na >> %var";

    assert_eq!(
        await_io! {
            LineByLineExecuter::new(NoLog::default(), get_b)
                .apply_fallible("a", rules)
        },
        Ok("b".to_string())
    );
}

#[io_test(pollster::block_on)]
fn line_by_line_log_runtime() {
    let rules = "PRINT 1:\na >> b\nPRINT 2:\nc >> d\nPRINT 3:";

    let mut executor = LineByLineExecuter::new(LogRuntime::default(), NoGet);

    assert_eq!(
        await_io! { executor.apply_fallible("abcd", rules) },
        Ok("bbdd".to_string())
    );

    assert_eq!(
        executor.runtime().logs(),
        &[
            ("1:".to_string(), "abcd".to_string()),
            ("2:".to_string(), "bbcd".to_string()),
            ("3:".to_string(), "bbdd".to_string()),
        ]
    );
}

struct RefContextLogger<'a>(PhantomData<&'a ()>);

impl<'a> ContextRuntime for RefContextLogger<'a> {
    type OutputContext = &'a mut Vec<(String, String)>;

    fn put_io(&mut self, context: Self::OutputContext, msg: &str, phones:String) -> Result<Self::OutputContext, String> {
        context.push((msg.to_string(), phones));
        Ok(context)
    }
}

struct RcContextLogger;

impl ContextRuntime for RcContextLogger {
    type OutputContext = Rc<RefCell<Vec<(String, String)>>>;

    fn put_io(&mut self, context: Self::OutputContext, msg: &str, phones:String) -> Result<Self::OutputContext, String> {
        context.borrow_mut().push((msg.to_string(), phones));
        Ok(context)
    }
}

#[io_test(pollster::block_on)]
fn context_runtimes() {
    let mut logs = Vec::new();
    let runtime = RefContextLogger(PhantomData);

    let mut executor = LineByLineExecuter::new(runtime, NoGet);
    executor.apply_with_contexts("pata", "PRINT this is a test:", &mut logs, ());

    assert_eq!(logs, vec![("this is a test:".to_string(), "pata".to_string())]);


    let mut logs = Vec::new();
    let mut runtime = RefContextLogger(PhantomData);

    let appliable_rules = build_rules("PRINT this is a test:", &mut NoGet).expect("should build");
    appliable_rules.apply_with_context("pata", &mut runtime, &mut logs);

    assert_eq!(logs, vec![("this is a test:".to_string(), "pata".to_string())]);


    let logs = Rc::new(RefCell::new(Vec::new()));
    let runtime = RcContextLogger;

    let mut executor = LineByLineExecuter::new(runtime, NoGet);
    executor.apply_with_contexts("pata", "PRINT this is a test:", logs.clone(), ());

    assert_eq!(logs.borrow().clone(), vec![("this is a test:".to_string(), "pata".to_string())]);


    let logs = Rc::new(RefCell::new(Vec::new()));
    let mut runtime = RcContextLogger;

    let appliable_rules = build_rules("PRINT this is a test:", &mut NoGet).expect("should build");
    appliable_rules.apply_with_context("pata", &mut runtime, logs.clone());

    assert_eq!(logs.borrow().clone(), vec![("this is a test:".to_string(), "pata".to_string())]);
}

struct RefContextGetter<'a>(PhantomData<&'a ()>);

impl<'a> ContextIoGetter for RefContextGetter<'a> {
    type InputContext = &'a mut dyn Iterator<Item = String>;

    fn get_io(&mut self, context: Self::InputContext, msg: &str) -> Result<(String, Self::InputContext), String> {
        context
            .next()
            .ok_or(format!("No input remains for prompt `{msg}`")).map(|msg| (msg, context))
    }
}

struct RcContextGetter;

impl ContextIoGetter for RcContextGetter {
    type InputContext = Rc<RefCell<dyn Iterator<Item = String>>>;

    fn get_io(&mut self, context: Self::InputContext, msg: &str) -> Result<(String, Self::InputContext), String> {
        let input = context
            .borrow_mut()
            .next();


        input.ok_or(format!("No input remains for prompt `{msg}`")).map(|msg| (msg, context))
    }
}

#[io_test(pollster::block_on)]
fn context_getters() {
    let mut inputs = ["in".to_string()].into_iter();
    let getter = RefContextGetter(PhantomData);

    let mut executor = LineByLineExecuter::new(NoLog::default(), getter);
    let res = executor.apply_with_contexts("a", "GET a test:\na >> %a", (), &mut inputs);

    assert_eq!(res, "in");


    let mut inputs = ["in".to_string()].into_iter();
    let mut getter = RefContextGetter(PhantomData);

    let appliable_rules = build_rules_with_context("GET a test:\na >> %a", &mut getter, &mut inputs).expect("should build");
    let res = appliable_rules.apply("a", &mut NoLog::default());

    assert_eq!(res, "in");

    
    let inputs = Rc::new(RefCell::new(["in".to_string()].into_iter()));
    let getter = RcContextGetter;

    let mut executor = LineByLineExecuter::new(NoLog::default(), getter);
    let res = executor.apply_with_contexts("a", "GET a test:\na >> %a", (), inputs);

    assert_eq!(res, "in");


    let inputs = Rc::new(RefCell::new(["in".to_string()].into_iter()));
    let mut getter = RcContextGetter;

    let appliable_rules = build_rules_with_context("GET a test:\na >> %a", &mut getter, inputs).expect("should build");
    let res = appliable_rules.apply("a", &mut NoLog::default());

    assert_eq!(res, "in");
}