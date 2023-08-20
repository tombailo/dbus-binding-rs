extern crate system_interface;
extern crate dbus;

use system_interface::Error8;
use system_interface::SystemService2::Signal0;
use system_interface::SystemService2::Signal1;
use system_interface::SystemService2::Signal2;
use system_interface::SystemService2::Signal3;
use system_interface::SystemService2::Interface as LA2;
use dbus::blocking::Connection;
use std::time::Duration;

fn call_method_5(la : &LA2)
{
    let ret_vals = la.method5().unwrap();

    for ret_val in ret_vals { println!("Val {:?}", ret_val); }
}

fn call_method_4(la : &LA2, method_arg : &String)
{
    match la.method4(method_arg) {
        Ok(val) => { println!("Value {:?}", val) },
        Err(err) => {
            if let Some(exception) = err.downcast_ref::<Error8>()
            {
                println!("{}", exception);
            }
            else
            {
                println!("Unexpected error: {}", err);
            }
        }
    }
}

fn main() {
    let connection = Connection::new_session().unwrap();

    let la = LA2::new(&connection, "Example.SystemService".to_string().into(), None);

    let mut cmd_line_args = std::env::args();
    let argv_0 = cmd_line_args.next().unwrap();

    if let Some(arg) = cmd_line_args.next()
    {
        if arg == "bookings"
        {
            call_method_5(&la);
        }
        else if arg == "booking"
        {
            if let Some(method_arg) = cmd_line_args.next()
            {
                call_method_4(&la, &method_arg);
            }
            else
            {
                eprintln!("Usage {} booking <booking ref>", argv_0);
            }
        }
        else if arg == "listen"
        {
            la.listenSignal0(|changes: Signal0| {
                println!("Signal0: {:?}", changes);
                true
            });

            la.listenSignal1(|changes: Signal1| {
                println!("Signal1: {:?}", changes);
                true
            });

            la.listenSignal2(|changes: Signal2| {
                println!("Signal2: {:?}", changes);
                true
            });

            la.listenSignal3(|warning: Signal3| {
                println!("Signal3: {:?}", warning);
                true
            });

            loop { connection.process(Duration::from_millis(1000)).unwrap(); }
        }
    }
    else
    {
        call_method_5(&la);

    }
}
