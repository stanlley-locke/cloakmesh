import re

with open('cloak-admin/src/App.tsx', 'r') as f:
    content = f.read()

# 1. Update layout state
old_layout_state = """  // Dynamic Layout & Persistence
  const [layout, setLayout] = useState<any[]>(() => {
    const saved = localStorage.getItem('cloak_dashboard_layout_v18');
    return saved ? JSON.parse(saved) : [{ i: 'AvailableHops', x: 0, y: 0, w: 2, h: 2 }, { i: 'AvgHSTime', x: 0, y: 2, w: 2, h: 2 }, { i: 'MeanHandlingTime', x: 2, y: 0, w: 4, h: 4 }, { i: 'NetworkStability', x: 6, y: 0, w: 6, h: 4 }, { i: 'RelayAudit', x: 0, y: 4, w: 8, h: 4 }];
  });"""

new_layout_state = """  // Dynamic Layout & Persistence
  const [layouts, setLayouts] = useState<Record<string, any[]>>(() => {
    const saved = localStorage.getItem('cloak_layouts_v19');
    if (saved) return JSON.parse(saved);
    return {
      dashboard: [{ i: 'AvailableHops_1', type: 'AvailableHops', x: 0, y: 0, w: 2, h: 2 }, { i: 'AvgHSTime_1', type: 'AvgHSTime', x: 0, y: 2, w: 2, h: 2 }, { i: 'MeanHandlingTime_1', type: 'MeanHandlingTime', x: 2, y: 0, w: 4, h: 4 }, { i: 'NetworkStability_1', type: 'NetworkStability', x: 6, y: 0, w: 6, h: 4 }, { i: 'RelayAudit_1', type: 'RelayAudit', x: 0, y: 4, w: 8, h: 4 }],
      identity: [{ i: 'IdentityView_1', type: 'IdentityView', x: 0, y: 0, w: 12, h: 8 }],
      messenger: [{ i: 'MessagingView_1', type: 'MessagingView', x: 0, y: 0, w: 12, h: 10 }],
      terminal: [{ i: 'TerminalView_1', type: 'TerminalView', x: 0, y: 0, w: 12, h: 12 }],
      discovery: [{ i: 'DiscoveryView_1', type: 'DiscoveryView', x: 0, y: 0, w: 12, h: 8 }],
      circuits: [{ i: 'CircuitsView_1', type: 'CircuitsView', x: 0, y: 0, w: 12, h: 12 }],
      dht: [{ i: 'DhtView_1', type: 'DhtView', x: 0, y: 0, w: 12, h: 12 }],
      capabilities: [{ i: 'CapabilitiesView_1', type: 'CapabilitiesView', x: 0, y: 0, w: 12, h: 10 }],
      encryption: [{ i: 'EncryptionLabView_1', type: 'EncryptionLabView', x: 0, y: 0, w: 12, h: 10 }],
      hosting: [{ i: 'HostingView_1', type: 'HostingView', x: 0, y: 0, w: 12, h: 10 }],
      settings: [{ i: 'SettingsView_1', type: 'SettingsView', x: 0, y: 0, w: 12, h: 8 }],
      hardware: [{ i: 'HardwareView_1', type: 'HardwareView', x: 0, y: 0, w: 12, h: 8 }],
      analytics: [{ i: 'AnalyticsView_1', type: 'AnalyticsView', x: 0, y: 0, w: 12, h: 12 }],
      browser: [{ i: 'BrowserView_1', type: 'BrowserView', x: 0, y: 0, w: 12, h: 14 }]
    };
  });

  const currentLayout = layouts[page] || [];

  const updateCurrentLayout = (newLayout: any[]) => {
    setLayouts(prev => ({ ...prev, [page]: newLayout }));
  };"""

content = content.replace(old_layout_state, new_layout_state)

# 2. Update the useEffects
content = content.replace("useEffect(() => { localStorage.setItem('cloak_dashboard_layout_v18', JSON.stringify(layout)); }, [layout]);",
                          "useEffect(() => { localStorage.setItem('cloak_layouts_v19', JSON.stringify(layouts)); }, [layouts]);")

# 3. Update addWidget function
old_add_widget = """  const addWidget = (module: any) => {
    const instanceId = `${module.id}_${Date.now()}`;
    const newWidget = { 
      i: instanceId, 
      type: module.id,
      x: (layout.length * 2) % 12, 
      y: 100, 
      w: module.w, 
      h: module.h 
    };
    setLayout((prev: any[]) => [...prev, newWidget]);
    setShowLibrary(false);
    setNotifications((prev: any[]) => [{ id: Date.now(), type: 'success', msg: `Provisioned ${module.title}`, time: 'Just now' }, ...prev]);
  };"""

new_add_widget = """  const addWidget = (module: any) => {
    const instanceId = `${module.id}_${Date.now()}`;
    const newWidget = { 
      i: instanceId, 
      type: module.id,
      x: (currentLayout.length * 2) % 12, 
      y: 100, 
      w: module.w, 
      h: module.h 
    };
    setLayouts((prev: any) => ({ ...prev, [page]: [...(prev[page] || []), newWidget] }));
    setShowLibrary(false);
    setNotifications((prev: any[]) => [{ id: Date.now(), type: 'success', msg: `Provisioned ${module.title}`, time: 'Just now' }, ...prev]);
  };"""

content = content.replace(old_add_widget, new_add_widget)

# 4. Update the main render area
old_render = """            {page === 'dashboard' ? (
              <ResponsiveGridLayout
                className="layout"
                layouts={{ lg: layout }}
                breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
                cols={{ lg: 12, md: 10, sm: 6, xs: 4, xxs: 2 }}
                rowHeight={60}
                width={containerWidth}
                isDraggable={isEditMode}
                isResizable={isEditMode}
                onLayoutChange={(newLayout: any) => setLayout(newLayout)}
              >
                {layout.map((w: any) => {
                  const moduleType = w.type || w.i.split('_')[0];
                  const catalogItem = WIDGET_CATALOG.find(c => c.id === moduleType);
                  return (
                    <div key={w.i}>
                      <Card 
                        title={catalogItem?.title || moduleType} 
                        subTitle={catalogItem?.subTitle}
                        isEditMode={isEditMode}
                        settings={widgetSettings[w.i]}
                        updateSettings={(s: any) => updateWidgetSettings(w.i, s)}
                        onRemove={() => setLayout((prev: any[]) => prev.filter((l: any) => l.i !== w.i))}
                      >
                        {Widgets[moduleType] ? React.createElement(Widgets[moduleType], { status, relays, searchQuery, filterStatus, settings: widgetSettings[w.i] }) : <div className="text-slate-300 italic text-[10px]">Component Missing</div>}
                      </Card>
                    </div>
                  );
                })}
              </ResponsiveGridLayout>
            ) : page === 'identity' ? (
              <IdentityView status={status} />
            ) : page === 'messenger' ? (
              <MessagingView />
            ) : page === 'terminal' ? (
              <TerminalView logs={structuredLogs} />
            ) : page === 'discovery' ? (
              <DiscoveryView status={status} />
            ) : page === 'circuits' ? (
              <CircuitsView />
            ) : page === 'dht' ? (
              <DhtView />
            ) : page === 'capabilities' ? (
              <CapabilitiesView />
            ) : page === 'encryption' ? (
              <EncryptionLabView />
            ) : page === 'hosting' ? (
              <HostingView />
            ) : page === 'settings' ? (
              <SettingsView />
            ) : page === 'hardware' ? (
              <HardwareView status={status} />
            ) : page === 'analytics' ? (
              <AnalyticsView />
            ) : page === 'browser' ? (
              <BrowserView />
            ) : ("""

new_render = """            {page ? (
              <ResponsiveGridLayout
                className="layout min-h-[500px]"
                layouts={{ lg: currentLayout }}
                breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
                cols={{ lg: 12, md: 10, sm: 6, xs: 4, xxs: 2 }}
                rowHeight={60}
                width={containerWidth}
                isDraggable={isEditMode}
                isResizable={isEditMode}
                onLayoutChange={(newLayout: any) => updateCurrentLayout(newLayout)}
              >
                {currentLayout.map((w: any) => {
                  const moduleType = w.type || w.i.split('_')[0];
                  const catalogItem = WIDGET_CATALOG.find(c => c.id === moduleType);
                  
                  const removeWidget = () => {
                    setLayouts((prev: any) => ({ ...prev, [page]: prev[page].filter((l: any) => l.i !== w.i) }));
                  };

                  const innerContent = Widgets[moduleType] ? React.createElement(Widgets[moduleType], { status, relays, searchQuery, filterStatus, settings: widgetSettings[w.i], logs: structuredLogs }) : <div className="text-slate-300 italic text-[10px]">Component Missing</div>;

                  return (
                    <div key={w.i} className="h-full">
                      {catalogItem?.isFullView ? (
                        <div className="h-full relative group w-full">
                          {isEditMode && (
                            <div className="absolute inset-0 bg-slate-900/10 z-50 flex items-center justify-center cursor-move border-2 border-dashed border-slate-400 rounded-3xl backdrop-blur-[1px]">
                              <div className="bg-white/90 p-4 rounded-2xl shadow-2xl flex flex-col items-center space-y-4 text-slate-900">
                                <Move size={32} className="animate-bounce" />
                                <button onMouseDown={(e) => { e.stopPropagation(); removeWidget(); }} className="p-2 bg-rose-500 text-white rounded-xl hover:bg-rose-600 transition-colors shadow-lg pointer-events-auto">
                                  <Trash2 size={16} />
                                </button>
                              </div>
                            </div>
                          )}
                          <div className={`h-full w-full ${isEditMode ? 'opacity-50 pointer-events-none' : ''}`}>
                             {innerContent}
                          </div>
                        </div>
                      ) : (
                        <Card 
                          title={catalogItem?.title || moduleType} 
                          subTitle={catalogItem?.subTitle}
                          isEditMode={isEditMode}
                          settings={widgetSettings[w.i]}
                          updateSettings={(s: any) => updateWidgetSettings(w.i, s)}
                          onRemove={removeWidget}
                          className="h-full w-full"
                        >
                          {innerContent}
                        </Card>
                      )}
                    </div>
                  );
                })}
              </ResponsiveGridLayout>
            ) : ("""

content = content.replace(old_render, new_render)

# 5. Move Sub-Page Views before Widgets and WIDGET_CATALOG
# Find the start of Widgets
idx_widgets = content.find("const Widgets: Record<string, any> = {")
# Find the end of BrowserView
idx_end_browser = content.find("// ── Main App Component ───────────────────────────────────────────────────────")

if idx_widgets != -1 and idx_end_browser != -1:
    idx_start_views = content.find("// ── Sub-Page Views ───────────────────────────────────────────────────────────")
    views_code = content[idx_start_views:idx_end_browser]
    
    # Remove views from original location
    content = content[:idx_start_views] + content[idx_end_browser:]
    
    # Insert views before Widgets
    content = content[:idx_widgets] + views_code + "\n\n" + content[idx_widgets:]

# 6. Update WIDGET_CATALOG and Widgets object
# We need to add the new views to the catalog and the Widgets object.

# Inject into Widgets
new_widgets_entries = """
  IdentityView: IdentityView,
  MessagingView: MessagingView,
  TerminalView: TerminalView,
  DiscoveryView: DiscoveryView,
  CircuitsView: CircuitsView,
  DhtView: DhtView,
  CapabilitiesView: CapabilitiesView,
  EncryptionLabView: EncryptionLabView,
  HostingView: HostingView,
  SettingsView: SettingsView,
  HardwareView: HardwareView,
  AnalyticsView: AnalyticsView,
  BrowserView: BrowserView,
"""
content = content.replace("const Widgets: Record<string, any> = {", "const Widgets: Record<string, any> = {" + new_widgets_entries)

# Inject into WIDGET_CATALOG
new_catalog_entries = """
  { id: 'IdentityView', title: 'IDENTITY_MANAGEMENT', subTitle: 'FULL_VIEW', w: 12, h: 8, isFullView: true },
  { id: 'MessagingView', title: 'DARKNET_MESSENGER', subTitle: 'FULL_VIEW', w: 12, h: 10, isFullView: true },
  { id: 'TerminalView', title: 'KERNEL_TERMINAL', subTitle: 'FULL_VIEW', w: 12, h: 12, isFullView: true },
  { id: 'DiscoveryView', title: 'PEER_DISCOVERY', subTitle: 'FULL_VIEW', w: 12, h: 8, isFullView: true },
  { id: 'CircuitsView', title: 'CIRCUIT_AUDIT', subTitle: 'FULL_VIEW', w: 12, h: 12, isFullView: true },
  { id: 'DhtView', title: 'DHT_EXPLORER', subTitle: 'FULL_VIEW', w: 12, h: 12, isFullView: true },
  { id: 'CapabilitiesView', title: 'PERMISSIONS', subTitle: 'FULL_VIEW', w: 12, h: 10, isFullView: true },
  { id: 'EncryptionLabView', title: 'CRYPTO_LAB', subTitle: 'FULL_VIEW', w: 12, h: 10, isFullView: true },
  { id: 'HostingView', title: 'SERVICE_HOSTING', subTitle: 'FULL_VIEW', w: 12, h: 10, isFullView: true },
  { id: 'SettingsView', title: 'PREFERENCES', subTitle: 'FULL_VIEW', w: 12, h: 8, isFullView: true },
  { id: 'HardwareView', title: 'HARDWARE_METRICS', subTitle: 'FULL_VIEW', w: 12, h: 8, isFullView: true },
  { id: 'AnalyticsView', title: 'ANALYTICS', subTitle: 'FULL_VIEW', w: 12, h: 12, isFullView: true },
  { id: 'BrowserView', title: 'DECENTRALIZED_BROWSER', subTitle: 'FULL_VIEW', w: 12, h: 14, isFullView: true },
"""
content = content.replace("const WIDGET_CATALOG = [", "const WIDGET_CATALOG = [" + new_catalog_entries)

# Fix TerminalView signature
content = content.replace("const TerminalView = ({ logs }: { logs: any[] }) => {", "const TerminalView = ({ logs }: any) => {")

with open('cloak-admin/src/App.tsx', 'w') as f:
    f.write(content)

print("App.tsx refactored successfully.")
