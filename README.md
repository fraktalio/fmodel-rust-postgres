# `fmodel-rust-postgres`

Effortlessly transform your domain models into powerful PostgreSQL extensions using our GitHub repository template.
With pre-implemented infrastructure and application layers in the `framework` module, you can focus entirely on your core domain logic while running your models directly within your PostgreSQL database for seamless integration and enhanced performance.

The template includes a demo domain model of a `restaurant/order management system`, showcasing practical implementation and providing a solid foundation for your own projects.

![event model](restaurant-model.jpg)

>Actually, the domain model is copied from the traditional application [fmodel-rust-demo](https://github.com/fraktalio/fmodel-rust-demo), demonstrating how to run your unique and single domain model directly within your PostgreSQL database/`as extension`; or connect the application to the database/`traditionally`.

## Event Sourcing

With event sourcing, we delve deeper by capturing every decision or alteration as an event.
Each new transfer or modification to the state is meticulously documented, providing a comprehensive audit trail
of all activities.
This affords you a 100% accurate historical record of your domain, enabling you to effortlessly traverse back
in time and review the state at any given moment.

**History is always on!**

## Technology
This project is using:

- [`rust` programming language](https://www.rust-lang.org/) to build a high-performance, reliable, and efficient system.
- [`f{model}` rust library](https://github.com/fraktalio/fmodel-rust) to implement tactical Domain-Driven Design patterns, optimised for Event Sourcing.
- [pgrx](https://github.com/pgcentralfoundation/pgrx) to simplify the creation of custom Postgres extensions and bring `logic` closer to your data(base).

## f{model}

[`f{model}`](https://github.com/fraktalio/fmodel-rust/blob/main/README.md) library provides just enough tactical Domain-Driven Design patterns, optimised for Event Sourcing and CQRS.

- algebraic data types form the structure of our data (commands, state, and events).
- functions/lambda offers the algebra of manipulating the data in a compositional manner, effectively modeling the behavior.

- This leads to modularity in design and a clear separation of the data’s structure and functions/behaviour of the data/entity.

`f{model}` library offers generic and abstract components to specialize in for your specific case/expected behavior.

[Read more](https://github.com/fraktalio/fmodel-rust/blob/main/README.md)



## Requirements
- [Rust](https://www.rust-lang.org/tools/install)
- [PGRX subcommand](https://github.com/pgcentralfoundation/pgrx?tab=readme-ov-file#getting-started): `cargo install --locked cargo-pgrx`
- Then you can run `cargo pgrx upgrade` in your extension's crate to update its dependencies.
- (Mac os) `brew install git icu4c pkg-config`
- (Mac os) `export PKG_CONFIG_PATH=/opt/homebrew/opt/icu4c/lib/pkgconfig`
- Run `cargo pgrx init` once, to properly configure the pgrx development environment. It downloads the latest releases of supported Postgres versions, configures them for debugging, compiles them with assertions, and installs them to `"${PGRX_HOME}"`. These include all contrib extensions and tools included with Postgres. Other cargo pgrx commands such as `run` and `test` will manage and use these installations on your behalf.

> No manual Postgres database installation is required.

## Test it
Run tests:

```shell
cargo pgrx test
```

## Run it
Compile/install extension to a pgrx-managed Postgres instance and start psql console:
```shell
cargo pgrx run
```

Now, you can run the following SQL commands in the psql console:

1. Load the extension:
```sql
create extension fmodel_rust_postgres;
```

2. Send commands to the system:

> Observe how the Commands are formatted in JSON format.

Create a restaurant:
```sql
select handle('{"type": "CreateRestaurant","identifier": "e48d4d9e-403e-453f-b1ba-328e0ce23737", "name": "Joe", "menu": {"menu_id": "02f09a3f-1624-3b1d-8409-44eff7708210", "items": [{"id": "02f09a3f-1624-3b1d-8409-44eff7708210","name": "supa","price": 10},{"id": "02f09a3f-1624-3b1d-8409-44eff7708210","name": "sarma","price": 20 }],"cuisine": "Vietnamese"}}'::Command);
```

Place an order at the restaurant:
```sql
select handle('{"type": "PlaceOrder","identifier": "e48d4d9e-403e-453f-b1ba-328e0ce23737", "order_identifier": "afd909c6-f8f3-49b2-af7f-833e933cbab4", "line_items": [{"id": "02f09a3f-1624-3b1d-8409-44eff7708210","quantity": 1, "menu_item_id": "02f09a3f-1624-3b1d-8409-44eff7708210", "name": "supa", "price": 10},{"id": "02f09a3f-1624-3b1d-8409-44eff7708210", "quantity": 1, "menu_item_id": "02f09a3f-1624-3b1d-8409-44eff7708210","name": "sarma","price": 20 }]}'::Command);
```


Confused? Run `cargo pgrx help`

## The structure of the project

The project is structured as follows:
- [lib.rs](src/lib.rs) file contains the entry point of the package/crate.
- `framework` module contains the generalized and parametrized implementation of infrastructure and application layers.
- `domain` module contains the domain model. It is the core and pure domain logic of the application!!!
- `application` module contains the application layer. It is the orchestration of the domain model and the infrastructure layer (empty, as it is implemented in the `framework` module).
- `infrastructure` module contains the infrastructure layer / fetching and storing data (empty, as it is implemented in the `framework` module).

The framework module offers a generic implementation of the infrastructure and application layers, which can be reused across multiple domain models.
Your focus should be on the `domain` module, where you can implement your unique domain model. We have provided a demo domain model of a `restaurant/order management system` to get you started.

## Check the tests
The project contains a set of tests that demonstrate how to use the domain model and the framework.
You can find them in the root: [lib.rs](src/lib.rs).

You will find a command handler function only, which can handle all the commands of the system! Simple!

## References and further reading
- [pgrx](https://github.com/pgcentralfoundation/pgrx)
- [fmodel-rust](https://github.com/fraktalio/fmodel-rust)

---
Created with :heart: by [Fraktalio](https://fraktalio.com/)
