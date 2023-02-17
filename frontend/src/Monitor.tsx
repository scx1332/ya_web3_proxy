import React, { useCallback, useContext, useState } from "react";
import "./Monitor.css";
import LatestCalls from "./LatestCalls";
import { BackendSettingsContext } from "./BackendSettingsProvider";
import { backendFetch } from "./common/BackendCall";

const Monitor = () => {
    const [nextRefresh, setNextRefresh] = useState(0);
    const { backendSettings } = useContext(BackendSettingsContext);
    const [keys, setKeys] = useState<string[]>([]);

    const loadActiveKeys = useCallback(async () => {
        try {
            const response = await backendFetch(backendSettings, `/keys/active`);
            const response_json = await response.json();
            setKeys(response_json.keys);
        } catch (e) {
            console.log(e);
            setKeys([]);
        }
    }, [setKeys]);

    React.useEffect(() => {
        console.log("Refreshing dashboard...");
        //timeout
        //sleep
        loadActiveKeys().then(() => {
            setTimeout(() => {
                setNextRefresh(nextRefresh + 1);
            }, 2000);
        });
    }, [setNextRefresh, nextRefresh]);

    function row(key: string) {
        return <LatestCalls key={key} apikey={key} refreshToken={nextRefresh} />;
    }

    return <div className={"monitor-appkey-lister"}>{keys.map(row)}</div>;
};

export default Monitor;
