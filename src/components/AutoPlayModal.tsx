import React, { useState } from 'react';
import {
  X,
  Play,
  Pause,
  Plus,
  Trash2,
  ChevronUp,
  ChevronDown,
  Bot,
  RotateCcw,
  Shield,
  MapPin,
} from 'lucide-react';
import {
  AutoPlayConfig,
  AutoLoop,
  AutoCard,
  AutoCondition,
  AutoAction,
  TurretDefenseLine,
} from '../bridge/wasmBridge';
import { TurretDefenseModal } from './TurretDefenseModal';

interface AutoPlayModalProps {
  isOpen: boolean;
  onClose: () => void;
  autoPlayConfig: AutoPlayConfig;
  onUpdateConfig: (newConfig: AutoPlayConfig) => void;
}

export const AutoPlayModal: React.FC<AutoPlayModalProps> = ({
  isOpen,
  onClose,
  autoPlayConfig,
  onUpdateConfig,
}) => {
  const [selectedLoopIndex, setSelectedLoopIndex] = useState<number>(0);
  const [isDefenseModalOpen, setIsDefenseModalOpen] = useState<boolean>(false);

  if (!isOpen) return null;

  const handleUpdateDefenseLine = (newLine: TurretDefenseLine) => {
    onUpdateConfig({
      ...autoPlayConfig,
      turret_defense: newLine,
    });
  };

  const activeLoop = autoPlayConfig.loops[selectedLoopIndex] || autoPlayConfig.loops[0];

  const handleToggleMaster = () => {
    onUpdateConfig({
      ...autoPlayConfig,
      enabled: !autoPlayConfig.enabled,
    });
  };

  const handleToggleAutoSupply = () => {
    onUpdateConfig({
      ...autoPlayConfig,
      auto_supply: !autoPlayConfig.auto_supply,
    });
  };

  const handleToggleLoop = (loopIdx: number) => {
    const newLoops = [...autoPlayConfig.loops];
    newLoops[loopIdx] = {
      ...newLoops[loopIdx],
      enabled: !newLoops[loopIdx].enabled,
    };
    onUpdateConfig({
      ...autoPlayConfig,
      loops: newLoops,
    });
  };

  const handleAddLoop = () => {
    const loopNum = autoPlayConfig.loops.length ? autoPlayConfig.loops.length + 1 : 1;
    const newLoop: AutoLoop = {
      id: `loop_custom_${Date.now()}`,
      name: `Vòng Lặp ${loopNum}`,
      enabled: true,
      current_card_index: 0,
      current_card_progress: 0,
      cards: [],
    };
    onUpdateConfig({
      ...autoPlayConfig,
      loops: [...autoPlayConfig.loops, newLoop],
    });
    setSelectedLoopIndex(autoPlayConfig.loops.length);
  };

  const handleDeleteLoop = (loopIdx: number) => {
    if (autoPlayConfig.loops.length <= 1) return;
    const newLoops = autoPlayConfig.loops.filter((_, idx) => idx !== loopIdx);
    onUpdateConfig({
      ...autoPlayConfig,
      loops: newLoops,
    });
    setSelectedLoopIndex(Math.max(0, loopIdx - 1));
  };

  const handleAddCard = () => {
    if (!activeLoop) return;
    const newCard: AutoCard = {
      id: `card_${Date.now()}`,
      name: 'Nhiệm vụ mới',
      enabled: true,
      condition: { type: 'Always' },
      action: {
        type: 'TrainUnit',
        unit_type: 'ShockTrooper',
        count: 1,
      },
    };
    const newLoops = [...autoPlayConfig.loops];
    newLoops[selectedLoopIndex] = {
      ...activeLoop,
      cards: [...activeLoop.cards, newCard],
    };
    onUpdateConfig({
      ...autoPlayConfig,
      loops: newLoops,
    });
  };

  const handleToggleCard = (cardIdx: number) => {
    if (!activeLoop) return;
    const newCards = [...activeLoop.cards];
    newCards[cardIdx] = {
      ...newCards[cardIdx],
      enabled: !newCards[cardIdx].enabled,
    };
    const newLoops = [...autoPlayConfig.loops];
    newLoops[selectedLoopIndex] = {
      ...activeLoop,
      cards: newCards,
    };
    onUpdateConfig({
      ...autoPlayConfig,
      loops: newLoops,
    });
  };

  const handleDeleteCard = (cardIdx: number) => {
    if (!activeLoop) return;
    const newCards = activeLoop.cards.filter((_, idx) => idx !== cardIdx);
    const newLoops = [...autoPlayConfig.loops];
    newLoops[selectedLoopIndex] = {
      ...activeLoop,
      cards: newCards,
      current_card_index: activeLoop.current_card_index >= newCards.length ? 0 : activeLoop.current_card_index,
    };
    onUpdateConfig({
      ...autoPlayConfig,
      loops: newLoops,
    });
  };

  const handleMoveCard = (cardIdx: number, direction: 'up' | 'down') => {
    if (!activeLoop) return;
    const targetIdx = direction === 'up' ? cardIdx - 1 : cardIdx + 1;
    if (targetIdx < 0 || targetIdx >= activeLoop.cards.length) return;
    const newCards = [...activeLoop.cards];
    const temp = newCards[cardIdx];
    newCards[cardIdx] = newCards[targetIdx];
    newCards[targetIdx] = temp;
    const newLoops = [...autoPlayConfig.loops];
    newLoops[selectedLoopIndex] = {
      ...activeLoop,
      cards: newCards,
    };
    onUpdateConfig({
      ...autoPlayConfig,
      loops: newLoops,
    });
  };

  const handleUpdateCardCondition = (cardIdx: number, newCond: AutoCondition) => {
    if (!activeLoop) return;
    const newCards = [...activeLoop.cards];
    newCards[cardIdx] = {
      ...newCards[cardIdx],
      condition: newCond,
    };
    const newLoops = [...autoPlayConfig.loops];
    newLoops[selectedLoopIndex] = {
      ...activeLoop,
      cards: newCards,
    };
    onUpdateConfig({
      ...autoPlayConfig,
      loops: newLoops,
    });
  };

  const handleUpdateCardAction = (cardIdx: number, newAction: AutoAction) => {
    if (!activeLoop) return;
    const newCards = [...activeLoop.cards];
    newCards[cardIdx] = {
      ...newCards[cardIdx],
      action: newAction,
    };
    const newLoops = [...autoPlayConfig.loops];
    newLoops[selectedLoopIndex] = {
      ...activeLoop,
      cards: newCards,
    };
    onUpdateConfig({
      ...autoPlayConfig,
      loops: newLoops,
    });
  };

  const handleResetDefaults = () => {
    const defaultCfg: AutoPlayConfig = {
      enabled: false,
      auto_supply: true,
      priority_techs: [],
      turret_defense: {
        enabled: true,
        p1: [520, 1680],
        p2: [860, 1980],
        spacing: 90,
        rows: 1,
        row_spacing: 80,
      },
      loops: [
        {
          id: 'loop_production',
          name: 'Sản Xuất',
          enabled: true,
          current_card_index: 0,
          current_card_progress: 0,
          cards: [
            {
              id: 'card_prod_worker',
              name: 'Mua 2 Thợ Mỏ (Engineer)',
              enabled: true,
              condition: { type: 'UnitCountLessThan', unit_type: 'Engineer', count: 7 },
              action: { type: 'TrainUnit', unit_type: 'Engineer', count: 2 },
            },
            {
              id: 'card_prod_reactor',
              name: 'Xây 2 PowerReactor',
              enabled: true,
              condition: { type: 'Always' },
              action: { type: 'BuildStructure', building_type: 'PowerReactor', count: 2 },
            },
            {
              id: 'card_prod_turret',
              name: 'Xây 2 TurretTower',
              enabled: true,
              condition: { type: 'Always' },
              action: { type: 'BuildStructure', building_type: 'TurretTower', count: 2 },
            },
            {
              id: 'card_prod_tech',
              name: 'Nâng cấp Cây Công Nghệ',
              enabled: true,
              condition: { type: 'Always' },
              action: {
                type: 'ResearchTech',
                tech_id: 'dark_matter_rockets',
                use_priority_queue: true,
              },
            },
            {
              id: 'card_prod_barracks',
              name: 'Xây 1 Barracks',
              enabled: true,
              condition: { type: 'Always' },
              action: { type: 'BuildStructure', building_type: 'Barracks', count: 1 },
            },
            {
              id: 'card_prod_converter',
              name: 'Xây EnergyConverter',
              enabled: true,
              condition: { type: 'ReactorPerConverterRatio' },
              action: { type: 'BuildStructure', building_type: 'EnergyConverter', count: 1 },
            },
          ],
        },
        {
          id: 'loop_military',
          name: 'Quân Sự',
          enabled: true,
          current_card_index: 0,
          current_card_progress: 0,
          cards: [
            {
              id: 'card_mil_shock',
              name: 'Tự động mua ShockTrooper',
              enabled: true,
              condition: { type: 'MineralsGreaterThan', amount: 500 },
              action: { type: 'TrainUnit', unit_type: 'ShockTrooper', count: 1 },
            },
            {
              id: 'card_mil_rifle',
              name: 'Tự động mua Rifleman',
              enabled: true,
              condition: { type: 'MineralsGreaterThan', amount: 500 },
              action: { type: 'TrainUnit', unit_type: 'Rifleman', count: 1 },
            },
            {
              id: 'card_mil_attack',
              name: 'Tự động tấn công (>40 lính)',
              enabled: true,
              condition: { type: 'UnitCountGreaterThan', unit_type: 'Combat', count: 40 },
              action: { type: 'AttackWave' },
            },
          ],
        },
      ],
    };
    onUpdateConfig(defaultCfg);
    setSelectedLoopIndex(0);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-4 select-none">
      <div className="bg-[#030d1a] border-2 border-emerald-500/80 rounded-xl w-full max-w-5xl shadow-[0_0_50px_rgba(16,185,129,0.3)] flex flex-col max-h-[92vh]">
        {/* Header Modal */}
        <div className="flex items-center justify-between px-6 py-3.5 border-b border-emerald-900/60 bg-slate-900/50">
          <div className="flex items-center gap-3">
            <Bot className="w-6 h-6 text-emerald-400 animate-pulse" />
            <div>
              <h2 className="text-base font-bold font-mono text-emerald-300 tracking-wider flex items-center gap-2">
                TACTICAL AUTO-PLAY BLUEPRINT // HỆ THỐNG VÒNG LẶP TỰ ĐỘNG
              </h2>
              <p className="text-xs text-slate-400">
                Thiết lập chuỗi nhiệm vụ và điều kiện tự động cho Sản xuất và Quân sự (Phím tắt: B)
              </p>
            </div>
          </div>

          <div className="flex items-center gap-4">
            {/* Master Toggle */}
            <button
              onClick={handleToggleMaster}
              className={`px-4 py-1.5 rounded-lg border font-mono text-xs font-bold flex items-center gap-2 transition-all cursor-pointer ${
                autoPlayConfig.enabled
                  ? 'bg-emerald-600/30 border-emerald-400 text-emerald-300 shadow-[0_0_12px_rgba(16,185,129,0.4)]'
                  : 'bg-slate-800 border-slate-700 text-slate-400 hover:text-slate-200'
              }`}
            >
              {autoPlayConfig.enabled ? (
                <>
                  <Play className="w-3.5 h-3.5 text-emerald-400 fill-emerald-400" />
                  <span>CHẾ ĐỘ TỰ ĐỘNG: ĐANG BẬT</span>
                </>
              ) : (
                <>
                  <Pause className="w-3.5 h-3.5 text-slate-400" />
                  <span>CHẾ ĐỘ TỰ ĐỘNG: ĐANG TẮT</span>
                </>
              )}
            </button>

            {/* Auto Supply Checkbox */}
            <label className="flex items-center gap-2 cursor-pointer select-none text-[11px] font-mono text-slate-300 bg-slate-950 px-3 py-1.5 rounded-lg border border-slate-700/80">
              <input
                type="checkbox"
                checked={autoPlayConfig.auto_supply}
                onChange={handleToggleAutoSupply}
                className="w-3.5 h-3.5 accent-emerald-400 cursor-pointer"
              />
              <span>Tự xây Supply tránh kẹt</span>
            </label>

            {/* Reset Defaults */}
            <button
              onClick={handleResetDefaults}
              className="p-1.5 text-slate-400 hover:text-amber-300 rounded-lg hover:bg-slate-800 transition-colors title='Khôi phục cấu hình mặc định'"
              title="Khôi phục mặc định"
            >
              <RotateCcw className="w-4 h-4" />
            </button>

            {/* Close */}
            <button
              onClick={onClose}
              className="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Danh sách các Vòng lặp (Loops Tabs) */}
        <div className="flex items-center justify-between border-b border-emerald-900/60 bg-slate-950/60 font-mono text-xs px-6 py-2 overflow-x-auto">
          <div className="flex items-center gap-2">
            {autoPlayConfig.loops.map((loop, idx) => {
              const isSelected = idx === selectedLoopIndex;
              return (
                <div
                  key={loop.id}
                  className={`flex items-center gap-2 px-3.5 py-1.5 rounded-lg border transition-all cursor-pointer ${
                    isSelected
                      ? 'bg-emerald-950/60 border-emerald-500 text-emerald-200 font-bold shadow-[0_0_12px_rgba(16,185,129,0.25)]'
                      : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:bg-slate-900 hover:text-slate-200'
                  }`}
                  onClick={() => setSelectedLoopIndex(idx)}
                >
                  <span
                    onClick={(e) => {
                      e.stopPropagation();
                      handleToggleLoop(idx);
                    }}
                    className={`w-2 h-2 rounded-full cursor-pointer ${
                      loop.enabled ? 'bg-emerald-400' : 'bg-slate-600'
                    }`}
                    title={loop.enabled ? 'Đang bật' : 'Đang tắt'}
                  />
                  <span>
                    {idx === 0 ? '🏭 ' : idx === 1 ? '⚔️ ' : '🔄 '}
                    {loop.name}
                  </span>
                  <span className="text-[10px] text-slate-500">
                    ({loop.cards.length} thẻ)
                  </span>
                  {autoPlayConfig.loops.length > 1 && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteLoop(idx);
                      }}
                      className="text-slate-500 hover:text-rose-400 ml-1"
                    >
                      ×
                    </button>
                  )}
                </div>
              );
            })}

            <button
              onClick={handleAddLoop}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-dashed border-slate-700 hover:border-emerald-500 text-slate-400 hover:text-emerald-300 font-mono text-xs transition-colors cursor-pointer"
            >
              <Plus className="w-3.5 h-3.5" />
              <span>+ Tạo Vòng Lặp Mới</span>
            </button>
          </div>

          <div className="flex items-center gap-4 text-[11px] font-mono">
            {/* Nút Mở Mini-Map Quy Hoạch Hàng Rào Phòng Thủ */}
            <button
              onClick={() => setIsDefenseModalOpen(true)}
              className="flex items-center gap-1.5 px-3 py-1 rounded-lg bg-amber-950/40 border border-amber-500/70 text-amber-300 hover:bg-amber-900/40 font-bold transition-all shadow-[0_0_10px_rgba(245,158,11,0.2)] cursor-pointer"
            >
              <Shield className="w-3.5 h-3.5 text-amber-400" />
              <span>🗺️ HÀNG RÀO PHÒNG THỦ TURRET</span>
              {autoPlayConfig.turret_defense?.enabled ? (
                <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse ml-0.5" />
              ) : (
                <span className="w-2 h-2 rounded-full bg-slate-500 ml-0.5" />
              )}
            </button>

            <div className="text-slate-400 flex items-center gap-2 border-l border-slate-800 pl-3">
              <span>Ưu tiên Tech Tree:</span>
              <span className="text-amber-400 font-bold">
                {autoPlayConfig.priority_techs.length > 0
                  ? `${autoPlayConfig.priority_techs.length} công nghệ ⭐`
                  : 'Chưa chọn'}
              </span>
            </div>
          </div>
        </div>

        {/* Nội dung danh sách Cards trong Vòng lặp đang chọn */}
        <div className="flex-1 overflow-y-auto p-6 space-y-3.5">
          {activeLoop && activeLoop.cards.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-12 text-slate-500 font-mono border border-dashed border-slate-800 rounded-xl">
              <Bot className="w-12 h-12 mb-3 text-slate-600" />
              <p>Vòng lặp này chưa có thẻ nhiệm vụ nào.</p>
              <button
                onClick={handleAddCard}
                className="mt-4 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded font-bold text-xs flex items-center gap-2 cursor-pointer"
              >
                <Plus className="w-4 h-4" /> Thêm Thẻ Nhiệm Vụ Đầu Tiên
              </button>
            </div>
          ) : (
            activeLoop?.cards.map((card, cardIdx) => {
              const isCurrent =
                autoPlayConfig.enabled &&
                activeLoop.enabled &&
                card.enabled &&
                activeLoop.current_card_index === cardIdx;

              return (
                <div
                  key={card.id}
                  className={`p-4 rounded-xl border transition-all ${
                    isCurrent
                      ? 'bg-emerald-950/40 border-emerald-400 ring-1 ring-emerald-400/50 shadow-[0_0_20px_rgba(16,185,129,0.3)]'
                      : card.enabled
                      ? 'bg-slate-900/60 border-slate-800 hover:border-slate-700'
                      : 'bg-slate-950/40 border-slate-900 opacity-50'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <span className="w-6 h-6 rounded-md bg-slate-800 text-emerald-400 flex items-center justify-center font-mono text-xs font-bold border border-slate-700">
                        #{cardIdx + 1}
                      </span>
                      <span className="font-mono text-sm font-bold text-slate-200">
                        {card.name}
                      </span>
                      {isCurrent && (
                        <span className="px-2 py-0.5 rounded bg-emerald-500/20 border border-emerald-400 text-emerald-300 font-mono text-[10px] font-bold animate-pulse">
                          ĐANG CHẠY ({activeLoop.current_card_progress} / {card.action.count || 1})
                        </span>
                      )}
                    </div>

                    <div className="flex items-center gap-1.5">
                      <button
                        onClick={() => handleMoveCard(cardIdx, 'up')}
                        disabled={cardIdx === 0}
                        className="p-1 text-slate-400 hover:text-white disabled:opacity-30 disabled:hover:text-slate-400"
                        title="Di chuyển lên"
                      >
                        <ChevronUp className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => handleMoveCard(cardIdx, 'down')}
                        disabled={cardIdx === activeLoop.cards.length - 1}
                        className="p-1 text-slate-400 hover:text-white disabled:opacity-30 disabled:hover:text-slate-400"
                        title="Di chuyển xuống"
                      >
                        <ChevronDown className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => handleToggleCard(cardIdx)}
                        className={`px-2 py-1 rounded font-mono text-[10px] font-bold ${
                          card.enabled
                            ? 'bg-emerald-950 border border-emerald-600 text-emerald-300'
                            : 'bg-slate-800 text-slate-500'
                        }`}
                      >
                        {card.enabled ? 'BẬT' : 'TẮT'}
                      </button>
                      <button
                        onClick={() => handleDeleteCard(cardIdx)}
                        className="p-1 text-slate-500 hover:text-rose-400 ml-2"
                        title="Xóa thẻ"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </div>

                  {/* Chi tiết Điều Kiện & Hành Động của Card */}
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-3 pt-3 border-t border-slate-800/80 font-mono text-xs">
                    {/* Cột 1: ĐIỀU KIỆN (Condition) */}
                    <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800">
                      <div className="text-[10px] text-slate-400 font-bold mb-1.5 flex items-center gap-1 text-amber-400">
                        <span>🔍 ĐIỀU KIỆN KÍCH HOẠT:</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <select
                          value={card.condition.type}
                          onChange={(e) => {
                            const t = e.target.value as any;
                            if (t === 'Always') {
                              handleUpdateCardCondition(cardIdx, { type: 'Always' });
                            } else if (t === 'UnitCountLessThan') {
                              handleUpdateCardCondition(cardIdx, {
                                type: 'UnitCountLessThan',
                                unit_type: 'Engineer',
                                count: 7,
                              });
                            } else if (t === 'UnitCountGreaterThan') {
                              handleUpdateCardCondition(cardIdx, {
                                type: 'UnitCountGreaterThan',
                                unit_type: 'Combat',
                                count: 40,
                              });
                            } else if (t === 'MineralsGreaterThan') {
                              handleUpdateCardCondition(cardIdx, {
                                type: 'MineralsGreaterThan',
                                amount: 500,
                              });
                            } else if (t === 'ReactorPerConverterRatio') {
                              handleUpdateCardCondition(cardIdx, {
                                type: 'ReactorPerConverterRatio',
                              });
                            }
                          }}
                          className="bg-slate-900 border border-slate-700 text-slate-200 text-xs rounded px-2 py-1 outline-none"
                        >
                          <option value="Always">Luôn thực hiện (Tuần tự)</option>
                          <option value="UnitCountLessThan">Số đơn vị &lt; X</option>
                          <option value="UnitCountGreaterThan">Số đơn vị &gt; X</option>
                          <option value="MineralsGreaterThan">Khoáng sản &gt; X</option>
                          <option value="ReactorPerConverterRatio">PowerReactor // 4 &gt; EnergyConverter</option>
                        </select>

                        {card.condition.type === 'UnitCountLessThan' && (
                          <div className="flex items-center gap-1">
                            <input
                              type="text"
                              value={card.condition.unit_type || 'Engineer'}
                              onChange={(e) =>
                                handleUpdateCardCondition(cardIdx, {
                                  ...card.condition,
                                  unit_type: e.target.value,
                                })
                              }
                              className="w-20 bg-slate-900 border border-slate-700 px-1.5 py-1 text-xs rounded"
                            />
                            <span>&lt;</span>
                            <input
                              type="number"
                              value={card.condition.count ?? 7}
                              onChange={(e) =>
                                handleUpdateCardCondition(cardIdx, {
                                  ...card.condition,
                                  count: parseInt(e.target.value) || 0,
                                })
                              }
                              className="w-14 bg-slate-900 border border-slate-700 px-1.5 py-1 text-xs rounded"
                            />
                          </div>
                        )}

                        {card.condition.type === 'UnitCountGreaterThan' && (
                          <div className="flex items-center gap-1">
                            <input
                              type="text"
                              value={card.condition.unit_type || 'Combat'}
                              onChange={(e) =>
                                handleUpdateCardCondition(cardIdx, {
                                  ...card.condition,
                                  unit_type: e.target.value,
                                })
                              }
                              className="w-20 bg-slate-900 border border-slate-700 px-1.5 py-1 text-xs rounded"
                            />
                            <span>&gt;</span>
                            <input
                              type="number"
                              value={card.condition.count ?? 40}
                              onChange={(e) =>
                                handleUpdateCardCondition(cardIdx, {
                                  ...card.condition,
                                  count: parseInt(e.target.value) || 0,
                                })
                              }
                              className="w-14 bg-slate-900 border border-slate-700 px-1.5 py-1 text-xs rounded"
                            />
                          </div>
                        )}

                        {card.condition.type === 'MineralsGreaterThan' && (
                          <div className="flex items-center gap-1">
                            <span>Khoáng &gt;</span>
                            <input
                              type="number"
                              value={card.condition.amount ?? 500}
                              onChange={(e) =>
                                handleUpdateCardCondition(cardIdx, {
                                  ...card.condition,
                                  amount: parseFloat(e.target.value) || 0,
                                })
                              }
                              className="w-16 bg-slate-900 border border-slate-700 px-1.5 py-1 text-xs rounded"
                            />
                          </div>
                        )}
                      </div>
                    </div>

                    {/* Cột 2: NHIỆM VỤ / HÀNH ĐỘNG (Action) */}
                    <div className="bg-slate-950/60 p-3 rounded-lg border border-slate-800">
                      <div className="text-[10px] text-slate-400 font-bold mb-1.5 flex items-center gap-1 text-cyan-400">
                        <span>⚡ NHIỆM VỤ KHI THỎA ĐIỀU KIỆN:</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <select
                          value={card.action.type}
                          onChange={(e) => {
                            const t = e.target.value as any;
                            if (t === 'TrainUnit') {
                              handleUpdateCardAction(cardIdx, {
                                type: 'TrainUnit',
                                unit_type: 'ShockTrooper',
                                count: 1,
                              });
                            } else if (t === 'BuildStructure') {
                              handleUpdateCardAction(cardIdx, {
                                type: 'BuildStructure',
                                building_type: 'PowerReactor',
                                count: 2,
                              });
                            } else if (t === 'ResearchTech') {
                              handleUpdateCardAction(cardIdx, {
                                type: 'ResearchTech',
                                tech_id: 'dark_matter_rockets',
                                use_priority_queue: true,
                              });
                            } else if (t === 'AttackWave') {
                              handleUpdateCardAction(cardIdx, { type: 'AttackWave' });
                            }
                          }}
                          className="bg-slate-900 border border-slate-700 text-slate-200 text-xs rounded px-2 py-1 outline-none"
                        >
                          <option value="TrainUnit">Mua Đơn Vị</option>
                          <option value="BuildStructure">Xây Công Trình</option>
                          <option value="ResearchTech">Nâng Cấp Công Nghệ</option>
                          <option value="AttackWave">Tự Động Tấn Công</option>
                        </select>

                        {card.action.type === 'TrainUnit' && (
                          <div className="flex items-center gap-1">
                            <select
                              value={card.action.unit_type || 'ShockTrooper'}
                              onChange={(e) =>
                                handleUpdateCardAction(cardIdx, {
                                  ...card.action,
                                  unit_type: e.target.value,
                                })
                              }
                              className="bg-slate-900 border border-slate-700 text-xs rounded px-2 py-1"
                            >
                              <option value="Engineer">Engineer (Thợ mỏ)</option>
                              <option value="ShockTrooper">ShockTrooper (Kiếm)</option>
                              <option value="Rifleman">Rifleman (Súng)</option>
                              <option value="Vanguard">Vanguard (Khiên)</option>
                              <option value="LaserMarksman">LaserMarksman</option>
                              <option value="DarkRocketeer">DarkRocketeer</option>
                            </select>
                            <span>SL:</span>
                            <input
                              type="number"
                              value={card.action.count ?? 1}
                              onChange={(e) =>
                                handleUpdateCardAction(cardIdx, {
                                  ...card.action,
                                  count: parseInt(e.target.value) || 1,
                                })
                              }
                              className="w-12 bg-slate-900 border border-slate-700 px-1 py-1 text-xs rounded"
                            />
                          </div>
                        )}

                        {card.action.type === 'BuildStructure' && (
                          <div className="flex items-center gap-1">
                            <select
                              value={card.action.building_type || 'PowerReactor'}
                              onChange={(e) =>
                                handleUpdateCardAction(cardIdx, {
                                  ...card.action,
                                  building_type: e.target.value,
                                })
                              }
                              className="bg-slate-900 border border-slate-700 text-xs rounded px-2 py-1"
                            >
                              <option value="PowerReactor">PowerReactor</option>
                              <option value="TurretTower">TurretTower</option>
                              <option value="Barracks">Barracks</option>
                              <option value="EnergyConverter">EnergyConverter</option>
                              <option value="SupplyDepot">SupplyDepot</option>
                            </select>
                            <span>SL:</span>
                            <input
                              type="number"
                              value={card.action.count ?? 1}
                              onChange={(e) =>
                                handleUpdateCardAction(cardIdx, {
                                  ...card.action,
                                  count: parseInt(e.target.value) || 1,
                                })
                              }
                              className="w-12 bg-slate-900 border border-slate-700 px-1 py-1 text-xs rounded"
                            />
                            {card.action.building_type === 'TurretTower' && (
                              <button
                                onClick={() => setIsDefenseModalOpen(true)}
                                className="px-2 py-0.5 rounded bg-amber-950 border border-amber-500/70 text-amber-300 text-[10px] font-bold flex items-center gap-1 hover:bg-amber-900/50 ml-1 cursor-pointer"
                                title="Mở bản đồ quy hoạch hàng rào tháp"
                              >
                                <MapPin className="w-3 h-3 text-amber-400" />
                                <span>Tuyến phòng thủ</span>
                              </button>
                            )}
                          </div>
                        )}

                        {card.action.type === 'ResearchTech' && (
                          <div className="flex items-center gap-1 text-[11px] text-amber-300">
                            <span>Ưu tiên Tech Tree ⭐ hoặc</span>
                            <input
                              type="text"
                              value={card.action.tech_id || 'dark_matter_rockets'}
                              onChange={(e) =>
                                handleUpdateCardAction(cardIdx, {
                                  ...card.action,
                                  tech_id: e.target.value,
                                })
                              }
                              className="w-32 bg-slate-900 border border-slate-700 px-1.5 py-1 text-xs rounded text-slate-100"
                            />
                          </div>
                        )}

                        {card.action.type === 'AttackWave' && (
                          <span className="text-[11px] text-rose-400 font-bold">
                            Tập kết lính và tổng tấn công căn cứ địch
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              );
            })
          )}

          {activeLoop && activeLoop.cards.length > 0 && (
            <button
              onClick={handleAddCard}
              className="w-full py-2.5 rounded-xl border border-dashed border-slate-700 hover:border-emerald-500 text-slate-400 hover:text-emerald-300 font-mono text-xs flex items-center justify-center gap-2 transition-all cursor-pointer bg-slate-900/30 hover:bg-slate-900/60"
            >
              <Plus className="w-4 h-4" /> Thêm Thẻ Nhiệm Vụ Vào Vòng Lặp
            </button>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-3 border-t border-emerald-900/40 bg-slate-950/80 flex justify-between items-center text-xs text-slate-400 font-mono">
          <div className="flex items-center gap-3">
            <span className="flex items-center gap-1.5">
              <span
                className={`w-2 h-2 rounded-full ${
                  autoPlayConfig.enabled ? 'bg-emerald-400 animate-ping' : 'bg-slate-500'
                }`}
              />
              <span className={autoPlayConfig.enabled ? 'text-emerald-300 font-semibold' : 'text-slate-400'}>
                {autoPlayConfig.enabled
                  ? 'Hệ thống tự động đang điều khiển chiến trường'
                  : 'Hệ thống tự động đang tắt (Điều khiển thủ công)'}
              </span>
            </span>
          </div>
          <button
            onClick={onClose}
            className="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded font-bold transition-all"
          >
            ĐÓNG [ESC / B]
          </button>
        </div>
      </div>

      {/* Modal Bản Đồ Nhỏ Quy Hoạch Hàng Rào Phòng Thủ */}
      <TurretDefenseModal
        isOpen={isDefenseModalOpen}
        onClose={() => setIsDefenseModalOpen(false)}
        defenseLine={
          autoPlayConfig.turret_defense || {
            enabled: true,
            p1: [520, 1680],
            p2: [860, 1980],
            spacing: 90,
            rows: 1,
            row_spacing: 80,
          }
        }
        onSave={handleUpdateDefenseLine}
      />
    </div>
  );
};
