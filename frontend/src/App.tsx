import React, { useState, useEffect, useCallback, useRef } from 'react';
import { Chessboard } from 'react-chessboard';
import { Chess } from 'chess.js';
import { w3cwebsocket as W3CWebSocket } from 'websocket';

interface GameState {
  fen: string;
  lastMove: string | null;
  gameOver: boolean;
  winner: string | null;
}

interface WebSocketMessage {
  type: string;
  move_str?: string;
  state?: GameState;
  message?: string;
}

function App() {
  const [game, setGame] = useState<Chess>(new Chess());
  const [gameState, setGameState] = useState<GameState>({
    fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
    lastMove: null,
    gameOver: false,
    winner: null,
  });
  const [logs, setLogs] = useState<string[]>([]);
  const [ws, setWs] = useState<W3CWebSocket | null>(null);
  const logsEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    logsEndRef.current?.scrollIntoView({ behavior: "smooth", block: "nearest" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [logs]);

  const addLog = useCallback((message: string) => {
    const timestamp = new Date().toISOString();
    setLogs(prev => [...prev, `[${timestamp}] ${message}`]);
  }, []);

  useEffect(() => {
    const client = new W3CWebSocket('ws://localhost:8080', undefined, undefined, {
      'Upgrade': 'websocket',
      'Connection': 'Upgrade',
      'Sec-WebSocket-Version': '13',
      'Sec-WebSocket-Key': 'your-key'
    });

    client.onopen = () => {
      addLog('Connected to game server');
    };

    client.onmessage = (message) => {
      if (typeof message.data === 'string') {
        try {
          const data: WebSocketMessage = JSON.parse(message.data);
          if (data.type === 'gameState' && data.state) {
            setGameState(data.state);
            const newGame = new Chess();
            newGame.load(data.state.fen);
            setGame(newGame);
            addLog(`Game state updated${data.move_str ? `: ${data.move_str}` : ' (Initial position)'}`);
          } else if (data.type === 'celestiaUpdate' && data.message) {
            addLog(data.message);
          }
        } catch (e) {
          addLog(`Error parsing message: ${e}`);
        }
      }
    };

    client.onclose = () => {
      addLog('Disconnected from game server');
    };

    client.onerror = (error) => {
      addLog(`WebSocket error: ${error.message || 'Connection failed'}`);
      setTimeout(() => {
        if (client.readyState === client.CLOSED) {
          client.close();
        }
      }, 1000);
    };

    setWs(client);

    return () => {
      client.close();
    };
  }, [addLog]);

  const onDrop = (sourceSquare: string, targetSquare: string) => {
    try {
      const move = game.move({
        from: sourceSquare,
        to: targetSquare,
        promotion: 'q',
      });

      if (move === null) return false;

      ws?.send(JSON.stringify({
        type: 'move',
        move_str: `${sourceSquare}${targetSquare}`,
      }));

      return true;
    } catch (error) {
      addLog(`Error making move: ${error}`);
      return false;
    }
  };

  return (
    <div className="min-h-screen bg-gray-100 py-6 flex flex-col justify-center sm:py-12">
      <div className="relative py-3 sm:max-w-xl sm:mx-auto">
        <div className="absolute inset-0 bg-gradient-to-r from-cyan-400 to-light-blue-500 shadow-lg transform -skew-y-6 sm:skew-y-0 sm:-rotate-6 sm:rounded-3xl"></div>
        <div className="relative px-4 py-10 bg-white shadow-lg sm:rounded-3xl sm:p-20">
          <div className="max-w-md mx-auto">
            <div className="divide-y divide-gray-200">
              <div className="py-8 text-base leading-6 space-y-4 text-gray-700 sm:text-lg sm:leading-7">
                <div className="w-[400px] h-[400px]">
                  <Chessboard 
                    position={gameState.fen}
                    onPieceDrop={onDrop}
                    boardWidth={400}
                  />
                </div>
                <div className="mt-4">
                  <h3 className="text-lg font-medium">Game Status</h3>
                  <p>{gameState.gameOver ? `Game Over - Winner: ${gameState.winner}` : `Current Turn: ${game.turn() === 'w' ? 'White' : 'Black'}`}</p>
                  <p>Last Move: {gameState.lastMove || 'None'}</p>
                </div>
                <div className="mt-4">
                  <h3 className="text-lg font-medium">Celestia Updates</h3>
                  <div className="h-40 overflow-y-auto bg-gray-50 p-2 rounded" style={{ scrollBehavior: 'smooth' }}>
                    {logs.map((log, index) => (
                      <div key={index} className="text-sm font-mono whitespace-pre-wrap">{log}</div>
                    ))}
                    <div ref={logsEndRef} style={{ float: 'left', clear: 'both' }} />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App; 