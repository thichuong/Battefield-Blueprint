import React, { useState, useEffect } from 'react';
import {
  X,
  Sliders,
  Shield,
  Building,
  Coins,
  FileCode,
  RotateCcw,
  Save,
  Download,
  Upload,
  Check,
  AlertTriangle,
} from 'lucide-react';
import { wasmBridge, GameConfig } from '../bridge/wasmBridge';
import defaultGameConfigData from '../config/defaultGameConfig.json';

interface GameConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfigApplied?: (config: GameConfig) => void;
}

type TabType = 'units' | 'buildings' | 'economy' | 'raw';

export const GAME_CONFIG_STORAGE_KEY = 'battlefield_blueprint_game_config';

export function loadSavedGameConfig(): GameConfig {
  try {
    const raw = localStorage.getItem(GAME_CONFIG_STORAGE_KEY);
    if (raw) {
      return JSON.parse(raw);
    }
  } catch (e) {
    console.warn('Failed to parse saved game config from localStorage, using defaults.', e);
  }
  return defaultGameConfigData as unknown as GameConfig;
}

export const GameConfigModal: React.FC<GameConfigModalProps> = ({
  isOpen,
  onClose,
  onConfigApplied,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('units');
  const [config, setConfig] = useState<GameConfig>(() => loadSavedGameConfig());
  const [selectedUnitKey, setSelectedUnitKey] = useState<string>('Engineer');
  const [selectedBuildingKey, setSelectedBuildingKey] = useState<string>('CommandCenter');
  const [rawJsonText, setRawJsonText] = useState<string>('');
  const [jsonError, setJsonError] = useState<string | null>(null);
  const [saveSuccessMsg, setSaveSuccessMsg] = useState<boolean>(false);

  // Sync state whenever modal opens
  useEffect(() => {
    if (isOpen) {
      const live = wasmBridge.getGameConfig();
      const current = live || loadSavedGameConfig();
      setConfig(current);
      setRawJsonText(JSON.stringify(current, null, 2));
      setJsonError(null);
      setSaveSuccessMsg(false);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const handleApplyConfig = (newCfg: GameConfig) => {
    const success = wasmBridge.setGameConfig(newCfg);
    if (success) {
      try {
        localStorage.setItem(GAME_CONFIG_STORAGE_KEY, JSON.stringify(newCfg));
      } catch (err) {
        console.error('Failed to save to localStorage:', err);
      }
      if (onConfigApplied) {
        onConfigApplied(newCfg);
      }
      setSaveSuccessMsg(true);
      setTimeout(() => setSaveSuccessMsg(false), 2000);
    } else {
      alert('Không thể áp dụng cấu hình vào Engine! Vui lòng kiểm tra lại giá trị.');
    }
  };

  const handleResetDefaults = () => {
    if (confirm('Bạn có chắc chắn muốn khôi phục toàn bộ thông số game về mặc định cân bằng ban đầu?')) {
      const reset = wasmBridge.resetGameConfig();
      const def = reset || (defaultGameConfigData as unknown as GameConfig);
      setConfig(def);
      setRawJsonText(JSON.stringify(def, null, 2));
      setJsonError(null);
      localStorage.removeItem(GAME_CONFIG_STORAGE_KEY);
      if (onConfigApplied) {
        onConfigApplied(def);
      }
      setSaveSuccessMsg(true);
      setTimeout(() => setSaveSuccessMsg(false), 2000);
    }
  };

  const handleExportJson = () => {
    const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(config, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute('download', 'battlefield_game_config.json');
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  };

  const handleImportJson = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const parsed = JSON.parse(event.target?.result as string);
        if (parsed.units && parsed.buildings && parsed.economy) {
          setConfig(parsed);
          setRawJsonText(JSON.stringify(parsed, null, 2));
          handleApplyConfig(parsed);
        } else {
          alert('File JSON không đúng định dạng GameConfig!');
        }
      } catch (err) {
        alert('Lỗi đọc file JSON: ' + (err as Error).message);
      }
    };
    reader.readAsText(file);
  };

  const updateSelectedUnit = (field: string, value: any) => {
    setConfig((prev) => {
      const updated = { ...prev };
      const currentUnit = updated.units[selectedUnitKey];
      if (currentUnit) {
        if (field.startsWith('res.')) {
          const resKey = field.split('.')[1] as keyof typeof currentUnit.resistances;
          currentUnit.resistances[resKey] = Math.max(0, Math.min(0.8, parseFloat(value) || 0));
        } else {
          (currentUnit as any)[field] = typeof (currentUnit as any)[field] === 'number' ? parseFloat(value) || 0 : value;
        }
      }
      setRawJsonText(JSON.stringify(updated, null, 2));
      return updated;
    });
  };

  const updateSelectedBuilding = (field: string, value: any) => {
    setConfig((prev) => {
      const updated = { ...prev };
      const currentB = updated.buildings[selectedBuildingKey];
      if (currentB) {
        if (field.startsWith('res.')) {
          const resKey = field.split('.')[1] as keyof typeof currentB.resistances;
          currentB.resistances[resKey] = Math.max(0, Math.min(0.8, parseFloat(value) || 0));
        } else {
          (currentB as any)[field] = typeof (currentB as any)[field] === 'number' ? parseFloat(value) || 0 : value;
        }
      }
      setRawJsonText(JSON.stringify(updated, null, 2));
      return updated;
    });
  };

  const updateEconomy = (field: keyof typeof config.economy, value: number) => {
    setConfig((prev) => {
      const updated = {
        ...prev,
        economy: {
          ...prev.economy,
          [field]: value,
        },
      };
      setRawJsonText(JSON.stringify(updated, null, 2));
      return updated;
    });
  };

  const selectedUnit = config.units[selectedUnitKey];
  const selectedBuilding = config.buildings[selectedBuildingKey];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-4">
      <div className="bg-[#030d1a] border-2 border-cyan-500/60 rounded-xl w-full max-w-4xl max-h-[90vh] shadow-2xl flex flex-col font-mono text-slate-200">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-900/60">
          <div className="flex items-center gap-3">
            <Sliders className="w-5 h-5 text-cyan-400" />
            <h2 className="text-base font-bold text-cyan-400 tracking-wider">
              GAME CONFIG // THÔNG SỐ TÙY CHỈNH
            </h2>
            {saveSuccessMsg && (
              <span className="flex items-center gap-1 text-xs text-emerald-400 bg-emerald-950/60 border border-emerald-500/50 px-2 py-0.5 rounded">
                <Check className="w-3.5 h-3.5" /> Đã lưu & Áp dụng!
              </span>
            )}
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-slate-800 rounded text-slate-400 hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Navigation Tabs */}
        <div className="flex border-b border-slate-800 bg-slate-950/50 px-6 gap-2 text-xs">
          <button
            onClick={() => setActiveTab('units')}
            className={`flex items-center gap-2 py-3 px-4 font-bold border-b-2 transition-all ${
              activeTab === 'units'
                ? 'border-cyan-400 text-cyan-300 bg-cyan-950/30'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Shield className="w-4 h-4" /> ĐƠN VỊ LÍNH (UNITS)
          </button>
          <button
            onClick={() => setActiveTab('buildings')}
            className={`flex items-center gap-2 py-3 px-4 font-bold border-b-2 transition-all ${
              activeTab === 'buildings'
                ? 'border-cyan-400 text-cyan-300 bg-cyan-950/30'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Building className="w-4 h-4" /> CÔNG TRÌNH (BUILDINGS)
          </button>
          <button
            onClick={() => setActiveTab('economy')}
            className={`flex items-center gap-2 py-3 px-4 font-bold border-b-2 transition-all ${
              activeTab === 'economy'
                ? 'border-cyan-400 text-cyan-300 bg-cyan-950/30'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <Coins className="w-4 h-4" /> KINH TẾ (ECONOMY)
          </button>
          <button
            onClick={() => {
              setActiveTab('raw');
              setRawJsonText(JSON.stringify(config, null, 2));
            }}
            className={`flex items-center gap-2 py-3 px-4 font-bold border-b-2 transition-all ${
              activeTab === 'raw'
                ? 'border-cyan-400 text-cyan-300 bg-cyan-950/30'
                : 'border-transparent text-slate-400 hover:text-slate-200'
            }`}
          >
            <FileCode className="w-4 h-4" /> JSON RAW
          </button>
        </div>

        {/* Tab Content Body */}
        <div className="flex-1 overflow-y-auto p-6 text-sm">
          {/* TAB 1: UNITS */}
          {activeTab === 'units' && selectedUnit && (
            <div className="flex flex-col md:flex-row gap-6">
              {/* Unit Selection List */}
              <div className="w-full md:w-56 flex flex-col gap-1 border-r border-slate-800 pr-4">
                <span className="text-xs font-semibold text-slate-400 mb-2 uppercase tracking-wider">
                  Chọn đơn vị:
                </span>
                {Object.keys(config.units).map((key) => {
                  const u = config.units[key];
                  const isSel = selectedUnitKey === key;
                  return (
                    <button
                      key={key}
                      onClick={() => setSelectedUnitKey(key)}
                      className={`text-left px-3 py-2 rounded text-xs transition-colors flex items-center justify-between ${
                        isSel
                          ? 'bg-cyan-950 text-cyan-300 border border-cyan-500 font-bold'
                          : 'hover:bg-slate-800 text-slate-300'
                      }`}
                    >
                      <span className="truncate">{key}</span>
                      <span className="text-[10px] text-slate-500">
                        {u.cost_minerals}M
                      </span>
                    </button>
                  );
                })}
              </div>

              {/* Unit Stats Editor */}
              <div className="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="col-span-full border-b border-slate-800 pb-2 flex justify-between items-center">
                  <h3 className="font-bold text-cyan-300 text-base">{selectedUnit.name}</h3>
                  <span className="text-xs text-slate-400">Loại: {selectedUnitKey}</span>
                </div>

                {/* HP */}
                <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                  <label className="text-xs text-slate-400 block mb-1">Máu tối đa (HP):</label>
                  <input
                    type="number"
                    value={selectedUnit.hp}
                    onChange={(e) => updateSelectedUnit('hp', e.target.value)}
                    className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 focus:border-cyan-400 outline-none"
                  />
                </div>

                {/* Speed */}
                <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                  <label className="text-xs text-slate-400 block mb-1">Tốc độ di chuyển (Speed):</label>
                  <input
                    type="number"
                    value={selectedUnit.speed}
                    onChange={(e) => updateSelectedUnit('speed', e.target.value)}
                    className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 focus:border-cyan-400 outline-none"
                  />
                </div>

                {/* Damage */}
                <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                  <label className="text-xs text-slate-400 block mb-1">Sát thương đòn đánh (Damage):</label>
                  <input
                    type="number"
                    value={selectedUnit.attack_damage}
                    onChange={(e) => updateSelectedUnit('attack_damage', e.target.value)}
                    className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 focus:border-cyan-400 outline-none"
                  />
                </div>

                {/* Range */}
                <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                  <label className="text-xs text-slate-400 block mb-1">Tầm bắn / Đánh (Range):</label>
                  <input
                    type="number"
                    value={selectedUnit.attack_range}
                    onChange={(e) => updateSelectedUnit('attack_range', e.target.value)}
                    className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 focus:border-cyan-400 outline-none"
                  />
                </div>

                {/* Cooldown */}
                <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                  <label className="text-xs text-slate-400 block mb-1">Hồi chiêu đòn đánh (Cooldown s):</label>
                  <input
                    type="number"
                    step="0.1"
                    value={selectedUnit.attack_cooldown}
                    onChange={(e) => updateSelectedUnit('attack_cooldown', e.target.value)}
                    className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 focus:border-cyan-400 outline-none"
                  />
                </div>

                {/* Damage Type */}
                <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                  <label className="text-xs text-slate-400 block mb-1">Hệ Sát thương (Damage Type):</label>
                  <select
                    value={selectedUnit.damage_type}
                    onChange={(e) => updateSelectedUnit('damage_type', e.target.value)}
                    className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1.5 text-cyan-400 focus:border-cyan-400 outline-none"
                  >
                    <option value="Physical">Vật lý (Physical)</option>
                    <option value="Energy">Năng lượng (Energy)</option>
                    <option value="Explosive">Nổ diện rộng (Explosive)</option>
                    <option value="Arcane">Ma thuật (Arcane)</option>
                  </select>
                </div>

                {/* Costs & Training Time */}
                <div className="col-span-full grid grid-cols-3 gap-3 bg-slate-950/60 p-3 rounded border border-slate-800">
                  <div>
                    <label className="text-xs text-amber-400 block mb-1">Giá Khoáng sản (M):</label>
                    <input
                      type="number"
                      value={selectedUnit.cost_minerals}
                      onChange={(e) => updateSelectedUnit('cost_minerals', e.target.value)}
                      className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-amber-300 font-bold outline-none"
                    />
                  </div>
                  <div>
                    <label className="text-xs text-sky-400 block mb-1">Giá Năng lượng (E):</label>
                    <input
                      type="number"
                      value={selectedUnit.cost_energy}
                      onChange={(e) => updateSelectedUnit('cost_energy', e.target.value)}
                      className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-sky-300 font-bold outline-none"
                    />
                  </div>
                  <div>
                    <label className="text-xs text-slate-400 block mb-1">Thời gian tạo (s):</label>
                    <input
                      type="number"
                      step="0.5"
                      value={selectedUnit.training_time}
                      onChange={(e) => updateSelectedUnit('training_time', e.target.value)}
                      className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-slate-300 outline-none"
                    />
                  </div>
                </div>

                {/* Resistances */}
                <div className="col-span-full bg-slate-950/60 p-3 rounded border border-slate-800">
                  <label className="text-xs text-slate-400 block mb-2 font-bold">
                    Kháng sát thương (0.00 = 0%, 0.80 = 80% tối đa):
                  </label>
                  <div className="grid grid-cols-4 gap-3 text-xs">
                    <div>
                      <span className="text-slate-400 block">Vật lý:</span>
                      <input
                        type="number"
                        step="0.05"
                        min="0"
                        max="0.8"
                        value={selectedUnit.resistances.physical}
                        onChange={(e) => updateSelectedUnit('res.physical', e.target.value)}
                        className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-slate-300"
                      />
                    </div>
                    <div>
                      <span className="text-sky-400 block">Năng lượng:</span>
                      <input
                        type="number"
                        step="0.05"
                        min="0"
                        max="0.8"
                        value={selectedUnit.resistances.energy}
                        onChange={(e) => updateSelectedUnit('res.energy', e.target.value)}
                        className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-sky-300"
                      />
                    </div>
                    <div>
                      <span className="text-amber-400 block">Nổ:</span>
                      <input
                        type="number"
                        step="0.05"
                        min="0"
                        max="0.8"
                        value={selectedUnit.resistances.explosive}
                        onChange={(e) => updateSelectedUnit('res.explosive', e.target.value)}
                        className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-amber-300"
                      />
                    </div>
                    <div>
                      <span className="text-purple-400 block">Ma thuật:</span>
                      <input
                        type="number"
                        step="0.05"
                        min="0"
                        max="0.8"
                        value={selectedUnit.resistances.arcane}
                        onChange={(e) => updateSelectedUnit('res.arcane', e.target.value)}
                        className="w-full bg-slate-900 border border-slate-700 rounded px-2 py-1 text-purple-300"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB 2: BUILDINGS */}
          {activeTab === 'buildings' && selectedBuilding && (
            <div className="flex flex-col md:flex-row gap-6">
              {/* Building Selection List */}
              <div className="w-full md:w-56 flex flex-col gap-1 border-r border-slate-800 pr-4">
                <span className="text-xs font-semibold text-slate-400 mb-2 uppercase tracking-wider">
                  Chọn công trình:
                </span>
                {Object.keys(config.buildings).map((key) => {
                  const b = config.buildings[key];
                  const isSel = selectedBuildingKey === key;
                  return (
                    <button
                      key={key}
                      onClick={() => setSelectedBuildingKey(key)}
                      className={`text-left px-3 py-2 rounded text-xs transition-colors flex items-center justify-between ${
                        isSel
                          ? 'bg-cyan-950 text-cyan-300 border border-cyan-500 font-bold'
                          : 'hover:bg-slate-800 text-slate-300'
                      }`}
                    >
                      <span className="truncate">{key}</span>
                      <span className="text-[10px] text-slate-500">
                        {b.cost_minerals}M
                      </span>
                    </button>
                  );
                })}
              </div>

              {/* Building Stats Editor */}
              <div className="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="col-span-full border-b border-slate-800 pb-2 flex justify-between items-center">
                  <h3 className="font-bold text-cyan-300 text-base">{selectedBuilding.name}</h3>
                  <span className="text-xs text-slate-400">Loại: {selectedBuildingKey}</span>
                </div>

                {/* HP */}
                <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                  <label className="text-xs text-slate-400 block mb-1">Máu công trình (HP):</label>
                  <input
                    type="number"
                    value={selectedBuilding.hp}
                    onChange={(e) => updateSelectedBuilding('hp', e.target.value)}
                    className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 focus:border-cyan-400 outline-none"
                  />
                </div>

                {/* Costs */}
                <div className="bg-slate-900/40 p-3 rounded border border-slate-800 flex gap-2">
                  <div className="flex-1">
                    <label className="text-xs text-amber-400 block mb-1">Giá Khoáng sản (M):</label>
                    <input
                      type="number"
                      value={selectedBuilding.cost_minerals}
                      onChange={(e) => updateSelectedBuilding('cost_minerals', e.target.value)}
                      className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-amber-300 font-bold outline-none"
                    />
                  </div>
                  <div className="flex-1">
                    <label className="text-xs text-sky-400 block mb-1">Giá Năng lượng (E):</label>
                    <input
                      type="number"
                      value={selectedBuilding.cost_energy}
                      onChange={(e) => updateSelectedBuilding('cost_energy', e.target.value)}
                      className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-sky-300 font-bold outline-none"
                    />
                  </div>
                </div>

                {/* Turret Tower Specifics */}
                {selectedBuildingKey === 'TurretTower' && (
                  <div className="col-span-full bg-cyan-950/20 p-3 rounded border border-cyan-800/40 grid grid-cols-3 gap-3">
                    <div className="col-span-full text-xs font-bold text-cyan-300">
                      THÔNG SỐ PHÁO PHÒNG THỦ (TURRET STATS):
                    </div>
                    <div>
                      <label className="text-xs text-slate-400 block mb-1">Tầm bắn:</label>
                      <input
                        type="number"
                        value={selectedBuilding.turret_range}
                        onChange={(e) => updateSelectedBuilding('turret_range', e.target.value)}
                        className="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-cyan-400"
                      />
                    </div>
                    <div>
                      <label className="text-xs text-slate-400 block mb-1">Sát thương:</label>
                      <input
                        type="number"
                        value={selectedBuilding.turret_damage}
                        onChange={(e) => updateSelectedBuilding('turret_damage', e.target.value)}
                        className="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-cyan-400"
                      />
                    </div>
                    <div>
                      <label className="text-xs text-slate-400 block mb-1">Hồi chiêu (s):</label>
                      <input
                        type="number"
                        step="0.1"
                        value={selectedBuilding.turret_cooldown}
                        onChange={(e) => updateSelectedBuilding('turret_cooldown', e.target.value)}
                        className="w-full bg-slate-950 border border-slate-700 rounded px-2 py-1 text-cyan-400"
                      />
                    </div>
                  </div>
                )}

                {/* Supply Depot Specifics */}
                {selectedBuildingKey === 'SupplyDepot' && (
                  <div className="col-span-full bg-slate-900/40 p-3 rounded border border-slate-800">
                    <label className="text-xs text-slate-400 block mb-1">Cộng thêm Giới hạn Dân số (Supply Bonus):</label>
                    <input
                      type="number"
                      value={selectedBuilding.supply_bonus}
                      onChange={(e) => updateSelectedBuilding('supply_bonus', e.target.value)}
                      className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 outline-none"
                    />
                  </div>
                )}

                {/* Power Reactor Specifics */}
                {selectedBuildingKey === 'PowerReactor' && (
                  <div className="col-span-full bg-slate-900/40 p-3 rounded border border-slate-800">
                    <label className="text-xs text-slate-400 block mb-1">Năng lượng sản sinh mỗi giây (Energy/s):</label>
                    <input
                      type="number"
                      step="0.5"
                      value={selectedBuilding.energy_per_sec}
                      onChange={(e) => updateSelectedBuilding('energy_per_sec', e.target.value)}
                      className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 outline-none"
                    />
                  </div>
                )}
              </div>
            </div>
          )}

          {/* TAB 3: ECONOMY */}
          {activeTab === 'economy' && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="col-span-full border-b border-slate-800 pb-2">
                <h3 className="font-bold text-cyan-300 text-base">THÔNG SỐ TÀI NGUYÊN & KHỞI ĐẦU</h3>
                <p className="text-xs text-slate-400">Điều chỉnh tài nguyên cấp khi vào trận và tốc độ đào quặng.</p>
              </div>

              {/* Starting Minerals */}
              <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                <label className="text-xs text-amber-400 block mb-1">Khoáng sản ban đầu (Starting Minerals):</label>
                <input
                  type="number"
                  value={config.economy.starting_minerals}
                  onChange={(e) => updateEconomy('starting_minerals', parseFloat(e.target.value) || 0)}
                  className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-amber-300 font-bold outline-none"
                />
              </div>

              {/* Starting Energy */}
              <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                <label className="text-xs text-sky-400 block mb-1">Năng lượng ban đầu (Starting Energy):</label>
                <input
                  type="number"
                  value={config.economy.starting_energy}
                  onChange={(e) => updateEconomy('starting_energy', parseFloat(e.target.value) || 0)}
                  className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-sky-300 font-bold outline-none"
                />
              </div>

              {/* Base Supply & Max Supply */}
              <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                <label className="text-xs text-slate-400 block mb-1">Giới hạn dân số ban đầu (Base Max Supply):</label>
                <input
                  type="number"
                  value={config.economy.base_max_supply}
                  onChange={(e) => updateEconomy('base_max_supply', parseInt(e.target.value, 10) || 0)}
                  className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 outline-none"
                />
              </div>

              {/* Base Energy Rate */}
              <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                <label className="text-xs text-slate-400 block mb-1">Năng lượng tự nhiên cơ bản (Base Energy/s):</label>
                <input
                  type="number"
                  step="0.5"
                  value={config.economy.base_energy_per_sec}
                  onChange={(e) => updateEconomy('base_energy_per_sec', parseFloat(e.target.value) || 0)}
                  className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 outline-none"
                />
              </div>

              {/* Worker Carry Capacity */}
              <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                <label className="text-xs text-slate-400 block mb-1">Sức vác khoáng sản thợ mỏ (Carry Capacity):</label>
                <input
                  type="number"
                  value={config.economy.worker_carry_capacity}
                  onChange={(e) => updateEconomy('worker_carry_capacity', parseFloat(e.target.value) || 0)}
                  className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 outline-none"
                />
              </div>

              {/* Worker Mining Time */}
              <div className="bg-slate-900/40 p-3 rounded border border-slate-800">
                <label className="text-xs text-slate-400 block mb-1">Thời gian cuốc 1 chuyến mỏ (Mining Time s):</label>
                <input
                  type="number"
                  step="0.1"
                  value={config.economy.worker_mining_time}
                  onChange={(e) => updateEconomy('worker_mining_time', parseFloat(e.target.value) || 0)}
                  className="w-full bg-slate-950 border border-slate-700 rounded px-2.5 py-1 text-cyan-400 outline-none"
                />
              </div>
            </div>
          )}

          {/* TAB 4: RAW JSON */}
          {activeTab === 'raw' && (
            <div className="flex flex-col h-full gap-2">
              <div className="flex justify-between items-center text-xs">
                <span className="text-slate-400">Bạn có thể chỉnh sửa trực tiếp JSON cấu hình:</span>
                {jsonError && (
                  <span className="text-rose-400 flex items-center gap-1">
                    <AlertTriangle className="w-3.5 h-3.5" /> {jsonError}
                  </span>
                )}
              </div>
              <textarea
                value={rawJsonText}
                onChange={(e) => {
                  const val = e.target.value;
                  setRawJsonText(val);
                  try {
                    const parsed = JSON.parse(val);
                    setConfig(parsed);
                    setJsonError(null);
                  } catch (err) {
                    setJsonError((err as Error).message);
                  }
                }}
                className="w-full h-80 bg-slate-950 font-mono text-xs text-emerald-400 p-3 border border-slate-700 rounded outline-none focus:border-cyan-400 resize-none"
              />
            </div>
          )}
        </div>

        {/* Footer Actions */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-slate-800 bg-slate-900/60">
          <div className="flex items-center gap-2">
            <button
              onClick={handleResetDefaults}
              className="flex items-center gap-1.5 px-3 py-2 rounded bg-slate-800 hover:bg-slate-700 text-xs font-bold text-slate-300 transition-colors"
            >
              <RotateCcw className="w-3.5 h-3.5" /> Mặc định
            </button>
            <button
              onClick={handleExportJson}
              className="flex items-center gap-1.5 px-3 py-2 rounded bg-slate-800 hover:bg-slate-700 text-xs font-bold text-slate-300 transition-colors"
            >
              <Download className="w-3.5 h-3.5" /> Xuất file
            </button>
            <label className="flex items-center gap-1.5 px-3 py-2 rounded bg-slate-800 hover:bg-slate-700 text-xs font-bold text-slate-300 transition-colors cursor-pointer">
              <Upload className="w-3.5 h-3.5" /> Nhập file
              <input
                type="file"
                accept=".json"
                onChange={handleImportJson}
                className="hidden"
              />
            </label>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 rounded bg-slate-800 hover:bg-slate-700 text-xs font-bold text-slate-300 transition-colors"
            >
              Đóng
            </button>
            <button
              onClick={() => handleApplyConfig(config)}
              className="flex items-center gap-2 px-5 py-2 rounded bg-cyan-600 hover:bg-cyan-500 text-xs font-bold text-white shadow-lg shadow-cyan-900/30 transition-colors"
            >
              <Save className="w-4 h-4" /> Lưu & Áp dụng ngay
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

