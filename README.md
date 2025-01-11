# Simulation Controller
## Functionality
This component orchestrates the interaction between clients servers and drones. The main tasks of the controller are: 
* parsing and validating the configuration file
* opening the channels and storing them for distribution among the nodes
* initializing the network nodes
* instantiating a network-wide topology based on the connections present in the configuration
* starting off the individual threads of each node
* interfacing with the tauri application to direct the network based on the commands from the front end 

## How to use
Import the libraries:

```rust
rustafarian-controller = { git = "https://github.com/Rustafarian-Unitn/rustafarian-controller", branch = "main" }
rustafarian-shared = { git = "https://github.com/Rustafarian-Unitn/rustafarian-shared.git" }
```

### Init
Import the library and the commands into the project
```rust
use rustafarian_controller::SimulationController;
use rustafarian_shared::messages::commander_messages::SimControllerCommand;
let debug: bool = false;
```
Initialize the component
```rust
let controller = SimulationController::build("config.toml", debug); 
```

### Sending a command to a drone
Retrieve the drone command channel
```rust
let drone_id = 1;
let drone_command_channel = &controller.drone_channels.get(&drone_id: NodeId)
    .unwrap()
    .receive_command_channel
    .unwrap();
```

Send a command
```rust
  let command = DroneCommand::RemoveSender(node_id: NodeId);
  drone_command_channel.send(command).unwrap();
```

### Sending a command to a node

Retrieve the drone command channel
```rust
let node_id = 1;
let client_channels = controller.nodes_channels.get(&node_id).unwrap();
```

Send the command
```rust
let client_command_channel = &client_channels.send_command_channel;
let command = SimControllerCommand::RequestServerType(server_id: NodeId);
client_command_channel.send(command).unwrap();
```

## List of available commands

### Node Commands

| Command | Description |
|---------|-------------|
| `SimControllerCommand::Register(server_id: NodeId)` | Instruct a Chat Client to register to the server identified by the server_id. Registered chat clients can communicate with each other through the server. |
| `SimControllerCommand::FloodRequest` | Instruct a client to initiate a flood request |
| `SimControllerCommand::RequestMediaFile(media_id: u8, server_id: NodeId)` | Instruct a client to retrieve the media identified by media_id from the server identified by server_id |
| `SimControllerCommand::RequestTextFile(media_id: u8, server_id: NodeId)` | Instruct a client to retrieve the text file identified by media_id from the server identified by server_id |
| `SimControllerCommand::RequestFileList(server_id: NodeId)` | Instruct a client to retrieve the file list from the server identified by server_id |
| `SimControllerCommand::SendMessage(message: String, server_node_id, client_node_id)` | Instruct the client to send a message to another client identified by client_node_id through the server identified by server_id |
| `SimControllerCommand::ClientList(server_id: NodeId)` | Instruct a client to retrieve the client list from the server identified by server_id |
| `SimControllerCommand::Topology` | Request a client the topology built from its latest topology discovery |
| `SimControllerCommand::KnownServers` | Request a client the list of known servers gathered from the last topology discovery |
| `SimControllerCommand::RegisteredServers` | Request a Chat Client the list of servers to which it is registered |
| `SimControllerCommand::AddSender(target_node_id: NodeId, sender_channel: NodeId)` | Add a crossbeam channel to the client. This action instantiates a new directed edge in the node's topology |
| `SimControllerCommand::RemoveSender(victim_id: NodeId)` | Remove the crossbeam channel of the victim node. This action removes a directed edge from the node's topology |

### Drone Commands

| Command | Description |
|---------|-------------|
| `DroneCommand::AddSender(target_node_id: NodeId, self_send_channel: Sender<Packet>)` | Add a new communication channel to the target node |
| `DroneCommand::Crash` | Simulate a drone crash by stopping its operation |
| `DroneCommand::RemoveSender(node_id: NodeId)` | Remove the communication channel to the specified node |
| `DroneCommand::SetPacketDropRate(pdr: f32)` | Set the packet drop rate for simulating network conditions |

### Controller Messages and Events

#### Node Events
These events are emitted when nodes perform actions in the network:

| Event | Description |
|-------|-------------|
| `MessageSent(session_id: u64)` | Triggered when a message is sent through a server to another node |
| `ChatMessageSent(server_id: NodeId, node_to: NodeId, message: String)` | Triggered when a message is sent through a Chat client send the message to another node |
| `FloodRequestSent` | Triggered when a node initiates a flood request |
| `PacketForwarded` | Triggered when a node forwards a packet in the network |

#### Node Messages 
These messages are responses from nodes to controller requests:

| Message | Description |
|---------|-------------|
| `FloodResponse(session_id: u64)` | Response to a flood request with session identifier |
| `TopologyResponse(topology: rustafarian_shared::Topology)` | Returns the current network topology view |
| `ClientListResponse(server_id: NodeId, client_list: Vec<NodeId>)` | Returns list of clients connected to a server |
| `MessageReceived(server_id: NodeId, node_from: NodeId, message: String)` | Indicates receipt of a message from another node |
| `TextFileResponse(file_id: u8, text: String)` | Returns requested text file content |
| `MediaFileResponse(file_id: u8, media: Vec<u8>)` | Returns requested media file content |
| `TextWithReferences(file_id: u8, text: String, references: Vec<u8>)` | Returns text file with associated ids of referenced files |
| `FileListResponse(server_id: NodeId, file_list: Vec<u8>)` | Returns list of files available on a server |
| `KnownServers(known_servers: Vec<NodeId>)` | Returns list of servers known to the node |
| `RegisteredServersResponse(registered_servers: Vec<NodeId>)` | Returns list of servers the node is registered to |
| `ServerTypeResponse(server_id: NodeId, server_type:ServerType)` | Returns the type of the specified server |

#### Drone Events
Events from drones related to packet handling:

| Event | Description |
|-------|-------------|
| `PacketSent(packet: wg_2024::packet::Packet)` | Triggered when a drone successfully sends a packet |
| `PacketDropped(packet: wg_2024::packet::Packet)` | Triggered when a drone drops a packet |
| `ControllerShortcut(packet: wg_2024::packet::Packet)` | Triggered when a drone uses a controller shortcut |


## Logging
The controller implements a simple logging feature that prints to the command line where the process is launched. The flag passed during its build phase determines the logging level used by the controller and the  nodes managed. If the flag passed is '''false''' then the logging level is INFO, otherwise the level is DEBUG with a much richer description of the interactions taking place in the network. 
