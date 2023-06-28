# a tiny entity-component-system library

* less dependencies

* fast to build

* easy to use

## Hello World

``` rust
// a function-system
fn hello_world() {
    println!("Hello World")
}

fn main() {
    // create a world
    let mut world = tecs::World::new();
    world
    // add "hello world" system into world's start_up_systems
    // startup_systems will only run once
    .add_startup_system(hello_world)
    // make the loop run once only
    .run_until(|| true);
}

```

## add components into world

you can use `spawn` method to add any types that implemented `Bundle` or `Component` trait into world

``` rust
use tecs::tools::Command;

let mut world = tecs::World::new();
world.spawn(12345);
world.spawn("abcde");
```
you can derive `Bundle` and `Component` trait easily

`Component` is just a tag, it could be implemented for any type

``` rust
use tecs::bundle::Component;

#[derive(Component)]
struct MyComponent{
    name : String,
}

```
all of the field of `Bundle` should implement `Component`

``` rust
use tecs::bundle::Bundle;

#[derive(Bundle)]
struct MyBundle{
    inner : MyComponent,
}

```

`Component` defaults to the following type implementation

* usize,u8,u16,u32,u64
* isize,i8,i16,i32,i64
* (),bool,&'static str

`Bundle` defaults to all tuple implementations that contain only types that`Component` trait implemented

such as:

* (usize,&str)


## query components in world
use `Query` directly        
``` rust
use tecs::world::Query;
use tecs::tool::Command;

let mut world = tecs::World::new();
// add two `Bundle` into world     
world.spawn(12345);
world.spawn((54321,"abcde"));

// create a `Query` to get all i32'a refence
let query = Query::<&i32>::new(&mut world);
for item in query {
    println!("{item}")
}
// this code will print: 
// 12345
// 54321
```

or use system

``` rust
use tecs::world::{Query,Commands};
use tecs::tool::Command;

// use command to do spawn
fn do_spawn(mut commands: Commands){
    commands.spawn(12345);
    commands.spawn((54321,"abcde"));
}

// note: you cant use conflict Query in one system, or the program will panic when you add system into world
// a example of conflict Query: Query<&i32> and Query<&mut i32>
fn print_all_i23(mut query : Query<&i32>){
    for item in query {
        println!("{item}")
    }   
}



world.add_startup_syste(do_spawn);
world.add_startup_syste(print_all_i23);
world.run_until(||true);

```
`Query` is a type that receiver two generic 

```rust
pub struct Query<'a, F: WorldFetch, Q: WorldFilter = ()> {...}
```
### WorldFetch is used to fetch component in world

it could be the (im)mutable reference of Component,or a tuple that contains only WorldFetch

### WorldFilter is used to fetch bundle in world

it could be 

* All<Component> or All<(Component1,Component1,...)> to filter bundle that contains all of components 

* OneOf<Component> or OneOf<(Component1,Component1,...)> to filter bundle that contains at least one of components

* Not<Component> or Not<(Component1,Component1,...)> to filter bundle that doesnot contain any one of components

### Example

* `Query<&i32>` will query all components that contain `i32` component, and give immutable references of `i32` in iterator

* use `Query<&mut i32>` to let iterator give mutable references of `i32` in iterator

* use `Query<&i32,All<&str>>` will query all components that contain `i32` and `&str`, and give immutable references of `i32` in iterator

* use `Query<&i32,AnyOf<(&str,MyComponent)>>` will query all components that contain `i32` and one of `&str` and `MyComponent`, and give immutable references of `i32` in iterator

* use `Query<&i32,All<&str>>` will query all components that contain `i32` and dont contain `&str`, and give immutable references of `i32` in iterator

### Iterator

you can use for to get the result of the query
the type of `fetch` is WorldFetch
``` rust 
for fetch in query {}
```

you can use for with .into_eiter() method to get the result of the query,and the `Entity` of result

``` rust 
for e in query.into_eiter() {}
```

the type of `e` is `EBundle`
```rust
pub struct EBundle<'a, F: WorldFetch> {...}
```
you can deref `e` to get the result of query, use .entity() method to get the `Entity` of result

`Entity` could be used to remove the bundle that queried
``` rust
commands.remove(b.entity());
```

## resources

Resources are stored in the world type by type

```rust
use tecs::tools::ResManager;

let mut world = tecs::World::new();
// get resources usize, and init it to 1
world.get_res<usize>().get_or_init(||1);
assert_eq!(*world.get_res<usize>().get().unwrap(),1);
```

[with system](https://github.com/twhice/tecs/blob/main/tecs/examples/resources.rs)

## features: System

this feature is really useful

this feature is enabled by default

this feature allow you to run system in world to access resources and components

only function-system supported in this version

all types that implemented `SystemParm` trait could be the parm of the function-system

follow types impled `SystemParm` trait 

you can find them in `tecs::world` mod


| type | usage | note |
| --- | --- | --- |
| Res<T> | to get resources of type T in world | cant use same Res<T> in one system |
| Resources | to get any type of resources in world | cant use be used with any Res in one system|
| Query<F,Q> | to query components in world | cant use conflict query in one system, like Query<&T> and Query<&mut T>|
Commands | to add and remove bundle into world | use spawn_many() method to spawn many bundle with the same type quickly|

to run a system,you need to add system into world by using `.add_system()` method or `.add_startup_system()` method fist 

* all startup_systems will only run once
* systems run pre loop


to run systems in world,you can

* use `.startup()` method to run all startup_systems

* use `.run_once()` method to run all systems once(dont include startup_systems)

* use `.run()` method to run all systems many times, this method will not return.

* use `.run_until(f)` method to run all systems many times, the loop will be break when `f` return `true`;

## features: async

this feature is disabled by default

this feature allow you to run async function-system in world

this feature make you can and only can add async function-system into world, reason:
```
note: upstream crates may add a new impl of trait `std::future::Future` for type `()` in future versions
```

the '.startup' '.run_once' '.run' '.run_until' methods become asynchronous functions

