interface UCIInfo {
    nodes: number;
    depth: number;
    time: number;
    score: string;
    pv: string[];
};

function uciParse(input: string): void {
    const tokens: string[] = input.split(/\s+/);

    let info: UCIInfo = {
        nodes: -1,
        depth: -1,
        time: -1,
        score: "",
        pv: []
    };

    let i: number = 0;
    while (i < tokens.length) {
        const token = tokens[i];
        switch (token) {
            case "nodes":
                info["nodes"] = parseInt(tokens[i + 1]);
                i += 2;
                break;
            case "time":
                info["time"] = parseInt(tokens[i + 1]);
                i += 2;
                break;
            case "score":
                // score cp 20
                const scoreType = tokens[i + 1];
                const scoreValue = tokens[i + 2];
                info["score"] = `${scoreType},${scoreValue}`;
                i += 3;
                break;
            case "depth":
                info["depth"] = parseInt(tokens[i + 1]);
                i += 2;
                break;
            case "pv":
                info["pv"] = tokens.slice(i + 1);
                i = tokens.length - 1;
                break;
            default:
                i += 1;
                break;
        }
    }

    console.log(info);
}

function main(): void {
    const response: string = "info depth 5 seldepth 7 nodes 1819 time 11 score cp 35 pv e2e4 e7e5 d2d4 e5d4";
    console.log(response);
    uciParse(response);
}

window.addEventListener("load", main);
