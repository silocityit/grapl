import React, {
	useEffect,
	useState,
	useMemo,
	useCallback,
	// useRef,
} from "react";
import { ForceGraph2D } from "react-force-graph";
import { nodeFillColor, riskOutline } from "./graphVizualization/nodeStyling";
import {
	calcLinkParticleWidth,
	calcLinkColor,
} from "./graphVizualization/linkCalcs";
import { nodeSize } from "./graphVizualization/nodeCalcs";
import { mapLabel } from "./graphLayout/labels";
import { updateGraph } from "./graphUpdates/updateGraph";
import { Link, VizNode, VizGraph } from "../../types/CustomTypes";
import {
	GraphState,
	GraphDisplayState,
	GraphDisplayProps,
} from "../../types/GraphDisplayTypes";

type ClickedNodeState = VizNode | null;

const defaultGraphDisplayState = (
	lensName: string | null
): GraphDisplayState => {
	return {
		graphData: { index: {}, nodes: [], links: [] },
		curLensName: lensName,
	};
};

const defaultClickedState = (): ClickedNodeState => {
	return null;
};

const GraphDisplay = ({ lensName, setCurNode }: GraphDisplayProps) => {
	// const fgRef: any = useRef(); // fix graph to canvas
	const [state, setState] = useState(defaultGraphDisplayState(lensName));

	useEffect(() => {
		const interval = setInterval(async () => {
			if (lensName) {
				await updateGraph(lensName, state as GraphState, setState); // state is safe cast, check that lens name is not null
			}
		}, 2000);
		return () => {
			clearInterval(interval);
		};
	}, [lensName, state, setState]);

	const data = useMemo(() => {
		const graphData = state.graphData;

		return graphData;
	}, [state]);
	const [clickedNode, setClickedNode] = useState(defaultClickedState());

	const nodeStyling = useCallback(
		(node, ctx) => {
			node.fx = node.x;
			node.fy = node.y;
			ctx.save();

			const NODE_R = nodeSize(node, data);

			ctx.beginPath(); // add ring to highlight hovered & neighbor nodes
			ctx.arc(node.x, node.y, NODE_R * 1.4, 0, 2 * Math.PI, false);
			ctx.fillStyle =
				node === clickedNode ? "cyan" : riskOutline(node.risk_score); // hovered node || risk score outline
			ctx.fill();

			// Node color
			ctx.beginPath();
			ctx.arc(node.x, node.y, NODE_R * 1.2, 0, 2 * Math.PI, false); // risk
			ctx.fillStyle =
				node === clickedNode ? "#DEFF00" : nodeFillColor(node.dgraph_type[0]);
			ctx.fill();
			ctx.save();

			const label = node.nodeLabel;

			ctx.font = "50px Roboto";
			const fontSize = Math.min(98, NODE_R / ctx.measureText(label).width);
			ctx.font = `${fontSize + 5}px Roboto`;

			const textWidth = ctx.measureText(label).width;

			const labelBkgdDimensions = [textWidth, fontSize].map(
				(n) => n + fontSize * 0.2
			);

			ctx.fillStyle = "rgba(0, 0, 0, 0.8)";

			ctx.fillRect(
				node.x - labelBkgdDimensions[0] / 2, // x coordinate
				node.y - labelBkgdDimensions[1] - 2.75, // y coordinate
				labelBkgdDimensions[0] + 1.25, // rectangle width
				labelBkgdDimensions[1] + 5.5 // rectangle height
			);

			ctx.textAlign = "center";
			ctx.textBaseline = "middle";
			ctx.fillStyle = "#ffffff";
			ctx.fillText(label, node.x, node.y);
			ctx.save();
		},
		[data, clickedNode]
	);

	const linkStyling = (link: any, ctx: any) => {
		const MAX_FONT_SIZE = 8;
		const LABEL_NODE_MARGIN = 8 * 1.5;

		const start = link.source;
		const end = link.target;
		ctx.save();

		link.color = calcLinkColor(link, data);

		// ignore unbounded links
		if (typeof start !== "object" || typeof end !== "object") return;

		// Edge label positioning calculations
		const textPos = {
			x: start.x + (end.x - start.x) / 2,
			y: start.y + (end.y - start.y) / 2,
		};
		ctx.save();

		const relLink = { x: end.x - start.x, y: end.y - start.y };
		const maxTextLength =
			Math.sqrt(Math.pow(relLink.x, 2) + Math.pow(relLink.y, 2)) -
			LABEL_NODE_MARGIN * 8;

		let textAngle = Math.atan2(relLink.y, relLink.x);

		// Maintain label vertical orientation for legibility
		if (textAngle > Math.PI / 2) textAngle = -(Math.PI - textAngle);
		if (textAngle < -Math.PI / 2) textAngle = -(-Math.PI - textAngle);

		const label = mapLabel(link.name);

		// Estimate fontSize to fit in link length
		ctx.font = "50px Roboto";
		const fontSize = Math.min(
			MAX_FONT_SIZE,
			maxTextLength / ctx.measureText(label).width
		);
		ctx.font = `${fontSize + 5}px Roboto`;
		// let textWidth = ctx.measureText(label).width;
		// textWidth += Math.round(textWidth * 0.25);

		// Draw text label
		ctx.save();
		ctx.translate(textPos.x, textPos.y);
		ctx.rotate(textAngle);
		ctx.textAlign = "center";
		ctx.textBaseline = "middle";
		ctx.fillText(label, 0.75, 3); //Content, left/right, top/bottom
		ctx.restore();
	};

	return (
		<ForceGraph2D
			graphData={data}
			// ref={fgRef}
			nodeLabel={"nodeLabel"} // tooltip on hover, actual label is in nodeCanvasObject
			nodeCanvasObject={nodeStyling}
			nodeCanvasObjectMode={() => "after"}
			onNodeClick={(_node, ctx) => {
				const node = _node as VizNode;
				setCurNode(node);
				setClickedNode(node || null);
			}}
			onNodeDragEnd={(node) => {
				node.fx = node.x;
				node.fy = node.y;
			}}
			linkColor={(link) => calcLinkColor(link as Link, data as VizGraph)}
			linkWidth={(link) => 7}
			linkDirectionalArrowLength={10}
			linkDirectionalArrowRelPos={1}
			linkDirectionalParticleSpeed={0.005}
			linkDirectionalParticleColor={(link) => "#919191"}
			linkDirectionalParticles={1}
			linkDirectionalParticleWidth={(link) =>
				calcLinkParticleWidth(link as Link, data as VizGraph) + 1
			}
			linkCanvasObjectMode={() => "after"}
			linkCanvasObject={linkStyling}
			warmupTicks={100}
			cooldownTicks={100}
		/>
	);
};

export default GraphDisplay;
