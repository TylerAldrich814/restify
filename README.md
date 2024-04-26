# restify!

A procedural macro for generating client-side code for REST APIs. Restify simplifies development by accelerating and automating
the creation of client-side RESTful API interactions, generating the essential boilerplate code typically required for
communicating with HTTP RESTful servers

## Basic Usage:
```rust
restify! {
    [pub MyUserEndpoint: {
        GET "/api/user/{id}" => {
            enum ErrorKind {
                Internal(String),
                Authentication(?String),
                NotFound(?String),
                Unkown,
            },
            ["camelCase"]
            struct Response {
                ["isError"]
                error: ?ErrorKind,
                user_name: ?String,
                email: ?String,
            },
            struct Query {
                id: String,
            }
        },
        POST "/api/user/{id}/message" => {
            struct Header {
                auth: String,
                ["Secret"]
                sec: Vec<u8>,
                date_time: DateTime,
            },
            struct Request {
                id: String,
                message: String,
            },
            struct Response {
                ["isError"]
                error: ErrorKind,
                message_id: ?String,
            }
        }
    }],
    [pub MyOtherEndpoint: {
        //...
    }]
}
```

The expected syntax I've developed for the `restify!` Macro incovation almost matches Rust's Syntax, all but a few key differences.
- **Struct Declarations**: Currently, the parser is designed to recognize specific struct names, although this may be subject to future changes. This feature was developed based on the observation that these particular data types often exhibit similar patterns across various REST APIs that I have developed clients for. Each of these structs includes built-in function implementations that are compiled based on both the *uri* and *parameters* the user specifies within restify!. This approach ensures that the functionality is tailored to effectively manage API interactions.

    - **Header**
        * **Purpose**: Facilitates the handling of critical HTTP header data both inbound and outbound.
        * **Functionality**: Implements both Serialize and Deserialize to allow easy manipulation of header information, such as authentication tokens, CORS settings, and other necessary metadata. This dual capability ensures headers can be both read from incoming requests and set for outgoing responses, supporting functionalities like authentication signatures and managing cross-origin resource sharing.
    - **Request**
        * **Purpose**: Manages data being sent to the server in API requests.
        * **Functionality**: Implements Serialize to convert request data into a suitable format for HTTP transmission. This struct is crucial for encapsulating the body of outgoing requests, ensuring that data such as JSON or form parameters are correctly serialized according to the API specifications.
    - **Response**
        * **Purpose**: Handles data received from the server in API responses.
        * **Functionality**: Implements Deserialize to parse incoming response data into structured Rust types. This allows clients to easily work with data returned from APIs, facilitating error handling, data validation, and further processing within client applications.
    - **ReqRes**
        * **Purpose**: Serves dual roles in handling both requests and responses, useful in scenarios where the same data structure is transmitted and received.
        * **Functionality(()): Implements both Serialize and Deserialize, making it ideal for APIs where the request and response entities are identical or very similar, reducing the need for separate structs and streamlining the codebase.
    - **Query**
        * **Purpose**: Manages URL query parameters, often used in GET requests or to supplement POST requests.
        * **Functionality**: Implements Serialize to efficiently convert query parameters into URL-encoded strings. This struct ensures that all query parameters are correctly formatted and appended to URLs, supporting complex querying capabilities like pagination, filtering, and sorting.


* **Derives**: Curently, I do not have a parser in place to allow specific macro declarations for either structs or enum. At the moment, the compiled code will automatically derive Debug, and  *serde::Serialize* or *serde::Deserialize* depending on which struct variant you choose(Adding this is in my future features list).
* **Optional Values**: To create an Optional value in either a Struct or in an enum, you add a  '?' at the begininng of the Type declaration.
    - ```user_name: ?String``` *will compile to* ```user_name: Option<String>```
    -  Defining a type as optional will also trigger the compiler to include specific Serde Attributes, depending on which struct variant the parameter is in.
    - If defined in a Serializable struct, then the parameter  `#[serde(skip_serializing_if="Option::is_none")]` will be added to the compiled code.
    - If defined in a Deserializable struct, then `#[default]` wil be added to the compiled code.

* **serde's `rename` & `rename_all` attributes**: Currently, picking which attirbute depends on where you place it.
    - Placing `["CamelCase"]` above either an enum or struct declaration will be parsed into `#[serde(rename_all="CamelCase")]`.
    - Placing `["UserID"]` above a parameter declaration will be parsed into `#[serde(rename="UserID")]` with that particular parameter.

# `restify!`'s Current Status:
At the moment, *restify!* is able to dynamically parse the syntax example from above, and will generate the mod/struct/enum definitions. There's still a lot more to to get this macro to a
point where you can drop it in a project, write out your client code and instantly connect to any
REST API.

## Features:
- [x]   Dynamically Parses *M* Endpoints, Each with *N* REST  Methods containing *K* structs/enums with *J* parameters.
- [x]   Compiles parsed data into custom modules, each with the REST method functionalites defined within restify!.
- [x]   Compiles Documentation for each compiled struct.
- [x]   Serde Attribute parser / Serde Attribute compiler.
#### In Development
- [ ]   Global Declarations: Enum/Struct Definitions - Defined above the Endpoint defintions. Creating a global 'Header' struct.
- [ ]   Developing the actual REST Client logic for making REST method requests.
- [ ]   Custom Compile Errors, using syn's built in features.
- [ ]   Adding features to define it the Client should be Synchronous or Asynchronous.
- [ ]   Once we have the basic functionalites developed for making API calls, creating suitable unit tests for each step of the parsing and compiling logic.
- [ ]   using *restify!* as a build step, instead of purely integrating it in a project. i.e., Using restify! within build.rs. Where the user would define the file name & module for the compiled code to be generated into. Adding a hashing mechanisms similar to git for detecting and re-compiling if any changes have been detected.
