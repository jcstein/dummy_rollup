import React, { useState, useEffect } from 'react';
import { Chessboard } from 'react-chessboard';
import { Chess } from 'chess.js';
import WebSocket from 'websocket';

interface GameState {
  fen: string;
  lastMove: string | null;
  gameOver: boolean;
  winner: string | null;
}

function App() {
  const [game, setGame] = useState(new Chess());
  const [gameState, setGameState] = useState<GameState>({
    fen: 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1',
    lastMove: null,
    gameOver: false,
    winner: null,
  });
  const [logs, setLogs] = useState<string[]>([]);
  const [ws, setWs] = useState<WebSocket.w3cwebsocket | null>(null);

  useEffect(() => {
    const client = new WebSocket.w3cwebsocket('ws://localhost:8080');

    client.onopen = () => {
      addLog('Connected to game server');
    };

    client.onmessage = (message) => {
      if (typeof message.data === 'string') {
        const data = JSON.parse(message.data);
        if (data.type === 'gameState') {
          setGameState(data.state);
          game.load(data.state.fen);
          addLog(`Game state updated: ${data.state.lastMove || 'Initial position'}`);
        } else if (data.type === 'celestiaUpdate') {
          addLog(data.message);
        }
      }
    };

    client.onclose = () => {
      addLog('Disconnected from game server');
    };

    setWs(client);

    return () => {
      client.close();
    };
  }, []);

  const addLog = (message: string) => {
    const timestamp = new Date().toISOString();
    setLogs(prev => [...prev, `[${timestamp}] ${message}`]);
  };

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
        move: `${sourceSquare}${targetSquare}`,
      }));

      return true;
    } catch (error) {
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
                  />
                </div>
                <div className="mt-4">
                  <h3 className="text-lg font-medium">Game Status</h3>
                  <p>{gameState.gameOver ? `Game Over - Winner: ${gameState.winner}` : `Current Turn: ${game.turn() === 'w' ? 'White' : 'Black'}`}</p>
                  <p>Last Move: {gameState.lastMove || 'None'}</p>
                </div>
                <div className="mt-4">
                  <h3 className="text-lg font-medium">Celestia Updates</h3>
                  <div className="h-40 overflow-y-auto bg-gray-50 p-2 rounded">
                    {logs.map((log, index) => (
                      <div key={index} className="text-sm font-mono">{log}</div>
                    ))}
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