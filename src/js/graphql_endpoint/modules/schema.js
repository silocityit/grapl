const dgraph = require("dgraph-js");
const grpc = require("@grpc/grpc-js");
const { GraphQLJSONObject } = require("graphql-type-json");
const { json } = require("express");

const AWS = require("aws-sdk");
const IS_LOCAL = (process.env.IS_LOCAL == 'True') || null;

// ## TODO? DECIDE HOW TO get an instance of dynamodb in graphql??
const getDisplayProperty = async (nodeType) => {
	const region = process.env.AWS_REGION;

	AWS.config.update({ region: region }); // is the env file the right place to get this value?
	// comes from local-grapl.env
	
	const ddb = new AWS.DynamoDB({ 
		apiVersion: "2012-08-10",
		region: IS_LOCAL ? process.env.AWS_REGION : undefined,
		accessKeyId: IS_LOCAL ? process.env.DYNAMODB_ACCESS_KEY_ID : undefined,
		secretAccessKey: IS_LOCAL ? process.env.DYNAMODB_ACCESS_KEY_SECRET : undefined,
		endpoint: IS_LOCAL ? process.env.DYNAMODB_ENDPOINT : undefined,
	}); // new client 

	const params = {
		TableName: process.env.GRAPL_DISPLAY_TABLE,
		Key: {
			node_type: { S: nodeType },// get display prop for a given node based on the type
		},
		ProjectionExpression: "display_name", // identifies	the attributes that you want to query for
	};

	// Call DynamoDB to read the item from the table
	const response = await ddb.getItem(params).promise(); 
	console.log("response from ddb", response)

	// if (err) {
	// 	console.log("Error", err);
	//   } else {
	// 	console.log("Success", data);
	//   }

};

const {
	GraphQLObjectType,
	GraphQLInt,
	GraphQLString,
	GraphQLList,
	GraphQLSchema,
	GraphQLUnionType,
	GraphQLNonNull,
	GraphQLBoolean,
} = require("graphql");

const BaseNode = {
	uid: { type: GraphQLInt },
	node_key: { type: GraphQLString },
	dgraph_type: { type: GraphQLList(GraphQLString) },
	display: {type: GraphQLString}
};

const LensNodeType = new GraphQLObjectType({
	name: "LensNode",
	fields: () => ({
		...BaseNode,
		lens_name: { type: GraphQLString },
		score: { type: GraphQLInt },
		scope: { type: GraphQLList(GraplEntityType) },
		lens_type: { type: GraphQLString },

	}),
});

const RiskType = new GraphQLObjectType({
	name: "Risk",
	fields: {
		...BaseNode,
		analyzer_name: { type: GraphQLString },
		risk_score: { type: GraphQLInt },
	},
});

// We have to support every type in grapl_analyzerlib/schemas
// We also have to support dynamic types, which would map to plugins,
// probably using the GraphQLJsonType

// TODO: File is missing all of its properties
const FileType = new GraphQLObjectType({
	name: "File",
	fields: {
		...BaseNode,
		file_name: { type: GraphQLString },
		file_type: { type: GraphQLString },
		file_extension: { type: GraphQLString },
		file_mime_type: { type: GraphQLString },
		file_size: { type: GraphQLInt },
		file_version: { type: GraphQLString },
		file_description: { type: GraphQLString },
		file_product: { type: GraphQLString },
		file_company: { type: GraphQLString },
		file_directory: { type: GraphQLString },
		file_inode: { type: GraphQLInt },
		file_hard_links: { type: GraphQLString },
		signed: { type: GraphQLBoolean },
		signed_status: { type: GraphQLString },
		md5_hash: { type: GraphQLString },
		sha1_hash: { type: GraphQLString },
		sha256_hash: { type: GraphQLString },
		risks: { type: GraphQLList(RiskType) },
		file_path: { type: GraphQLString },
	},
});

const IpConnections = new GraphQLObjectType({
	name: "IpConnections",
	fields: () => ({
		...BaseNode,
		risks: { type: GraphQLList(RiskType) },
		src_ip_addr: { type: GraphQLString },
		src_port: { type: GraphQLString },
		dst_ip_addr: { type: GraphQLString },
		dst_port: { type: GraphQLString },
		created_timestamp: { type: GraphQLInt },
		terminated_timestamp: { type: GraphQLInt },
		last_seen_timestamp: { type: GraphQLInt },
		inbound_ip_connection_to: { type: IpAddressType },
	}),
});

// TODO: Process is missing many properties and edges
// 'fields' is a callback, so that we can declare ProcessType first, and then
// reference it in 'children' later
// This is called lazy evaluation, where we defer the execution of code until it is needed
const ProcessType = new GraphQLObjectType({
	name: "Process",
	fields: () => ({
		...BaseNode,
		created_timestamp: { type: GraphQLInt },
		image_name: { type: GraphQLString },
		process_name: { type: GraphQLString },
		arguments: { type: GraphQLString },
		children: {
			type: GraphQLList(ProcessType),
		},
		bin_file: { type: FileType },
		created_file: { type: FileType },
		deleted_files: { type: FileType },
		read_files: { type: GraphQLList(FileType) },
		wrote_files: { type: GraphQLList(FileType) },
		created_connections: { type: GraphQLList(ProcessOutboundConnections) },
		inbound_connections: { type: GraphQLList(ProcessInboundConnections) },
		process_id: { type: GraphQLInt },
		risks: { type: GraphQLList(RiskType) },
	}),
});

const NetworkConnection = new GraphQLObjectType({
	name: "NetworkConnection",
	fields: () => ({
		src_ip_address: { type: GraphQLString },
		src_port: { type: GraphQLString },
		dst_ip_address: { type: GraphQLString },
		dst_port: { type: GraphQLString },
		created_timestamp: { type: GraphQLInt },
		terminated_timestamp: { type: GraphQLInt },
		last_seen_timestamp: { type: GraphQLInt },
		inbound_network_connection_to: { type: GraphQLList(IpPort) },
	}),
});

const IpPort = new GraphQLObjectType({
	name: "IpPort",
	fields: {
		...BaseNode,
		ip_address: { type: GraphQLString },
		protocol: { type: GraphQLString },
		port: { type: GraphQLInt },
		first_seen_timestamp: { type: GraphQLInt },
		last_seen_timestamp: { type: GraphQLInt },
		network_connections: { type: GraphQLList(NetworkConnection) },
	},
});

const IpAddressType = new GraphQLObjectType({
	name: "IpAddress",
	fields: {
		...BaseNode,
		risks: { type: GraphQLList(RiskType) },
		ip_address: { type: GraphQLString },
	},
});

const AssetType = new GraphQLObjectType({
	name: "Asset",
	fields: {
		...BaseNode,
		risks: { type: GraphQLList(RiskType) },
		hostname: { type: GraphQLString },
		asset_ip: { type: GraphQLList(IpAddressType) },
		asset_processes: { type: GraphQLList(ProcessType) },
		files_on_asset: { type: GraphQLList(FileType) },
	},
});

const ProcessInboundConnections = new GraphQLObjectType({
	name: "ProcessInboundConnections",
	fields: {
		...BaseNode,
		ip_address: { type: GraphQLString },
		protocol: { type: GraphQLString },
		created_timestamp: { type: GraphQLInt },
		terminated_timestamp: { type: GraphQLInt },
		last_seen_timestamp: { type: GraphQLInt },
		port: { type: GraphQLInt },
		bound_port: { type: GraphQLList(IpPort) },
		bound_ip: { type: GraphQLList(IpAddressType) },
	},
});

const ProcessOutboundConnections = new GraphQLObjectType({
	name: "ProcessOutboundConnections",
	fields: {
		...BaseNode,
		ip_address: { type: GraphQLString },
		protocol: { type: GraphQLString },
		created_timestamp: { type: GraphQLInt },
		terminated_timestamp: { type: GraphQLInt },
		last_seen_timestamp: { type: GraphQLInt },
		port: { type: GraphQLInt },
		connected_over: { type: GraphQLList(IpPort) },
		connected_to: { type: GraphQLList(IpPort) },
	},
});

const PluginType = new GraphQLObjectType({
	name: "PluginType",
	fields: {
		predicates: { type: GraphQLJSONObject },
	},
});

const builtins = new Set([
	"Process",
	"File",
	"IpAddress",
	"Asset",
	"Risk",
	"IpConnections",
	"ProcessInboundConnections",
	"ProcessOutboundConnections",
]);

// TODO: Handle the rest of the builtin types
const resolveType = (data) => {
	data.dgraph_type = data.dgraph_type.filter(
		(t) => t !== "Entity" && t !== "Base"
	);

	if (data.dgraph_type[0] === "Process") {
		return "Process";
	}

	if (data.dgraph_type[0] === "File") {
		return "File";
	}

	if (data.dgraph_type[0] === "IpAddress") {
		return "IpAddress";
	}

	if (data.dgraph_type[0] === "Asset") {
		return "Asset";
	}

	if (data.dgraph_type[0] === "Risk") {
		return "Risk";
	}

	if (data.dgraph_type[0] === "IpConnections") {
		return "IpConnections";
	}

	if (data.dgraph_type[0] === "ProcessInboundConnections") {
		return "ProcessInboundConnections";
	}

	if (data.dgraph_type[0] === "ProcessOutboundConnections") {
		return "ProcessOutboundConnections";
	}
	return "PluginType";
};

// | FileType, ProcessType, IpAddressType, AssetType, RiskType, IpConnections, ProcessInboundConnections, ProcessOutboundConnections
const GraplEntityType = new GraphQLUnionType({
	name: "GraplEntityType",
	types: [PluginType, FileType, ProcessType, AssetType],
	resolveType: resolveType,
});

const get_random = (list) => {
	return list[Math.floor(Math.random() * list.length)];
};

const mg_alpha = get_random(process.env.MG_ALPHAS.split(","));

const getDgraphClient = () => {
	const clientStub = new dgraph.DgraphClientStub(
		mg_alpha,
		grpc.credentials.createInsecure()
	);

	return new dgraph.DgraphClient(clientStub);
};

const getLenses = async (dg_client, first, offset) => {
	console.log("first, offset parameters in getLenses()", first, offset);

	const query = `
		query all($a: int, $b: int)
		{
			all(func: type(Lens), first: $a, offset: $b, orderdesc: score)
			{
				lens_name,
				score,
				node_key,
				uid,
				dgraph_type: dgraph.type,
				lens_type,
				scope {
					uid,
					node_key,
					dgraph_type: dgraph.type,
				}
			}
		}
	`;

	console.log("Creating DGraph txn in getLenses");

	const txn = dg_client.newTxn();

	try {
		console.log("Querying DGraph for lenses in getLenses");
		const res = await txn.queryWithVars(query, {
			$a: first.toString(),
			$b: offset.toString(),
		});
		console.log("Returning lens response from DGraph");
		return res.getJson()["all"];
	} catch (e) {
		console.error("Error in DGraph txn getLenses: ", e);
	} finally {
		console.log("Closing Dgraph Txn in getLenses");
		await txn.discard();
	}
};

const getLensSubgraphByName = async (dg_client, lens_name) => {
	const query = `
		query all($a: string, $b: first, $c: offset) {
			all(func: eq(lens_name, $a), first: 1) {
				uid,
				dgraph_type: dgraph.type,
				node_key,
				lens_name,
				lens_type,
				score,
				scope @filter(has(node_key)) {
					uid,
					dgraph_type: dgraph.type,
					expand(_all_) {
						uid,
						dgraph_type: dgraph.type,
						expand(_all_)
					}
				}
			}
		}
    `;

	console.log("Creating DGraph txn in getLensSubgraphByName");
	const txn = dg_client.newTxn();

	try {
		console.log("Querying DGraph in getLensSubgraphByName");
		const res = await txn.queryWithVars(query, { $a: lens_name });
		console.log("returning data from getLensSubGrapByName: ");
		return res.getJson()["all"][0];
	} catch (e) {
		console.error("Error in DGraph txn: getLensSubgraphByName", e);
	} finally {
		console.log("Closing dgraphtxn in getLensSubraphByName");
		await txn.discard();
	}
};

const filterDefaultDgraphNodeTypes = (node_type) => {
	return node_type !== "Base" && node_type !== "Entity";
};

const handleLensScope = async (parent, args) => {
	console.log("handleLensScope args: ", args);
	const dg_client = getDgraphClient();

	const lens_name = args.lens_name;
	console.log("handling lens name: ", lens_name);

	// grab the graph of lens, lens scope, and neighbors to nodes in-scope of the lens ((lens) -> (neighbor) -> (neighbor's neighbor))
	const lens_subgraph = await getLensSubgraphByName(dg_client, lens_name);
	console.log("retrieved subgraph in handleLensScope");

	lens_subgraph["uid"] = parseInt(lens_subgraph["uid"], 16);
	// if it's undefined/null, might as well make it an array
	lens_subgraph["scope"] = lens_subgraph["scope"] || [];

	// start enriching the nodes within the scope
	lens_subgraph["scope"].forEach(
		(neighbor) => (neighbor["uid"] = parseInt(neighbor["uid"], 16))
	);	
	lens_subgraph["scope"].forEach(
		(neighbor) =>
			(neighbor["dgraph_type"] = neighbor["dgraph_type"].filter(
				filterDefaultDgraphNodeTypes
			))
	);
	// No dgraph_type? Not a node; skip it!
	lens_subgraph["scope"] = lens_subgraph["scope"].filter(
		(neighbor) => neighbor["dgraph_type"].length > 0
	);

	// record the uids of all direct neighbors to the lens.
	// These are the only nodes we should keep by the end of this process.
	// We'll then try to get all neighbor connections that only correspond to these nodes
	const neighbor_uids = new Set(
		lens_subgraph["scope"].map((node) => node["uid"])
	);

	for (let neighbor of lens_subgraph["scope"]) {
		// neighbor of a lens neighbor
		for (const predicate in neighbor) {
			// we want to keep risks and enrich them at the same time
	

			if (predicate === "risks") {
				neighbor[predicate].forEach((risk_node) => {
					risk_node["uid"] = parseInt(risk_node["uid"], 16);

					if ("dgraph_type" in risk_node) {
						console.log("checking if dgraph_type in risk_node", risk_node);
						risk_node["dgraph_type"] = risk_node["dgraph_type"].filter(
							filterDefaultDgraphNodeTypes
						);
					}
				});

				// filter out nodes that don't have dgraph_types
				neighbor[predicate] = neighbor[predicate].filter(
					(node) => "dgraph_type" in node && !!node["dgraph_type"]
				);
				continue;
			}

			// If this edge is 1-to-many, we need to filter down the list to lens-neighbor -> lens-neighbor connections
			if (
				Array.isArray(neighbor[predicate]) &&
				neighbor[predicate] &&
				neighbor[predicate][0]["uid"]
			) {
				neighbor[predicate].forEach(
					(node) => (node["uid"] = parseInt(node["uid"], 16))
				);
				neighbor[predicate].forEach(
					(node) =>
						(node["dgraph_type"] = node["dgraph_type"].filter(
							filterDefaultDgraphNodeTypes
						))
				);
				neighbor[predicate] = neighbor[predicate].filter((second_neighbor) =>
					neighbor_uids.has(second_neighbor["uid"])
				);

				// If we filtered all the edges down, might as well delete this predicate
				if (neighbor[predicate].length === 0) {
					delete neighbor[predicate];
				}
			}
			// If this edge is 1-to-1, we need to determine if we need to delete the edge
			else if (
				typeof neighbor[predicate] === "object" &&
				neighbor[predicate]["uid"]
			) {
				if (!neighbor_uids.has(parseInt(neighbor[predicate]["uid"], 16))) {
					delete neighbor[predicate];
				} else {
					neighbor[predicate]["uid"] = parseInt(neighbor[predicate]["uid"], 16);
					neighbor[predicate]["dgraph_type"] = neighbor[predicate][
						"dgraph_type"
					].filter(filterDefaultDgraphNodeTypes);
				}
			}
		}
	}

	for (node of lens_subgraph["scope"]) {
		const displayProperty = await getDisplayProperty(nodeType.filter(filterDefaultDgraphNodeTypes));
		node["display"] = node[displayProperty].toString();
	}

	for (node of lens_subgraph["scope"]) {
		if (!builtins.has(node.dgraph_type[0])) {
			const tmpNode = { ...node };
			node.predicates = tmpNode;
		}
	}

	// for node in lens.scope pass node

	console.log("lens_subgraph scope", JSON.stringify(lens_subgraph["scope"]));
	return lens_subgraph;
};

const RootQuery = new GraphQLObjectType({
	name: "RootQueryType",
	fields: {
		lenses: {
			type: GraphQLList(LensNodeType),
			args: {
				first: {
					type: new GraphQLNonNull(GraphQLInt),
				},
				offset: {
					type: new GraphQLNonNull(GraphQLInt),
				},
			},
			resolve: async (parent, args) => {
				console.log("lenses query arguments", args);
				const first = args.first;
				const offset = args.offset;
				// #TODO: Make sure to validate that 'first' is under a specific limit, maybe 1000
				console.log("Making getLensesQuery");
				const lenses = await getLenses(getDgraphClient(), first, offset);
				console.debug(
					"returning data from getLenses for lenses resolver",
					lenses
				);
				return lenses;
			},
		},
		lens_scope: {
			type: LensNodeType,
			args: {
				lens_name: { type: new GraphQLNonNull(GraphQLString) },
			},
			resolve: async (parent, args) => {
				try {
					console.log("lens_scope args: ", args);
					let response = await handleLensScope(parent, args);
					console.log("lens_scope response: ", response);
					return await handleLensScope(parent, args);
				} catch (e) {
					console.error("Error in handleLensScope: ", e);
					throw e;
				}
			},
		},
	},
});

module.exports = new GraphQLSchema({
	query: RootQuery,
});
