//! Built-in dictionary for finite, enumerable finance terms.
//!
//! Sector names, security types, and officer titles form a small closed
//! vocabulary. Translating them through the ML backend is both slower and
//! less accurate than an exact lookup (e.g. NLLB renders "Technology" as a
//! full sentence in Japanese), so these terms are resolved here first.

use super::lang::Lang;

/// Languages covered by the built-in dictionary.
///
/// The discriminant indexes the translation rows below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DictLang {
    De = 0,
    Es = 1,
    Fr = 2,
    It = 3,
    Pt = 4,
    Nl = 5,
    Ja = 6,
    Ko = 7,
    ZhHans = 8,
    ZhHant = 9,
    Ru = 10,
}

impl DictLang {
    pub(crate) fn from_lang(lang: &Lang) -> Option<Self> {
        match (lang.primary(), lang.code().as_str()) {
            ("de", _) => Some(Self::De),
            ("es", _) => Some(Self::Es),
            ("fr", _) => Some(Self::Fr),
            ("it", _) => Some(Self::It),
            ("pt", _) => Some(Self::Pt),
            ("nl", _) => Some(Self::Nl),
            ("ja", _) => Some(Self::Ja),
            ("ko", _) => Some(Self::Ko),
            ("zh", "zh-Hant") => Some(Self::ZhHant),
            ("zh", _) => Some(Self::ZhHans),
            ("ru", _) => Some(Self::Ru),
            _ => None,
        }
    }
}

/// Look up an exact term in the built-in dictionary.
///
/// Row order: De, Es, Fr, It, Pt, Nl, Ja, Ko, ZhHans, ZhHant, Ru.
pub(crate) fn lookup(lang: DictLang, term: &str) -> Option<&'static str> {
    #[rustfmt::skip]
    let row: &[&'static str; 11] = match term {
        // ----- GICS sectors (Yahoo Finance `sectorDisp` values) -----
        "Technology" => &["Technologie", "Tecnología", "Technologie", "Tecnologia", "Tecnologia", "Technologie", "テクノロジー", "기술", "科技", "科技", "Технологии"],
        "Financial Services" => &["Finanzdienstleistungen", "Servicios financieros", "Services financiers", "Servizi finanziari", "Serviços financeiros", "Financiële dienstverlening", "金融サービス", "금융 서비스", "金融服务", "金融服務", "Финансовые услуги"],
        "Consumer Cyclical" => &["Zyklische Konsumgüter", "Consumo cíclico", "Consommation cyclique", "Beni di consumo ciclici", "Consumo cíclico", "Cyclische consumptiegoederen", "一般消費財", "임의 소비재", "周期性消费", "週期性消費", "Циклические потребительские товары"],
        "Consumer Defensive" => &["Defensive Konsumgüter", "Consumo defensivo", "Consommation défensive", "Beni di consumo difensivi", "Consumo defensivo", "Defensieve consumptiegoederen", "生活必需品", "필수 소비재", "防御性消费", "防禦性消費", "Защитные потребительские товары"],
        "Healthcare" => &["Gesundheitswesen", "Salud", "Santé", "Sanità", "Saúde", "Gezondheidszorg", "ヘルスケア", "헬스케어", "医疗保健", "醫療保健", "Здравоохранение"],
        "Industrials" => &["Industrie", "Industria", "Industrie", "Industria", "Indústria", "Industrie", "資本財", "산업재", "工业", "工業", "Промышленность"],
        "Energy" => &["Energie", "Energía", "Énergie", "Energia", "Energia", "Energie", "エネルギー", "에너지", "能源", "能源", "Энергетика"],
        "Basic Materials" => &["Grundstoffe", "Materiales básicos", "Matériaux de base", "Materiali di base", "Materiais básicos", "Basismaterialen", "素材", "소재", "基础材料", "基礎材料", "Сырьевые материалы"],
        "Real Estate" => &["Immobilien", "Bienes raíces", "Immobilier", "Immobiliare", "Imobiliário", "Vastgoed", "不動産", "부동산", "房地产", "房地產", "Недвижимость"],
        "Utilities" => &["Versorgungsunternehmen", "Servicios públicos", "Services publics", "Servizi di pubblica utilità", "Serviços públicos", "Nutsbedrijven", "公益事業", "유틸리티", "公用事业", "公用事業", "Коммунальные услуги"],
        "Communication Services" => &["Kommunikationsdienste", "Servicios de comunicación", "Services de communication", "Servizi di comunicazione", "Serviços de comunicação", "Communicatiediensten", "通信サービス", "통신 서비스", "通信服务", "通信服務", "Коммуникационные услуги"],
        // ----- Security types (`typeDisp` values) -----
        "Equity" => &["Aktie", "Acción", "Action", "Azione", "Ação", "Aandeel", "株式", "주식", "股票", "股票", "Акция"],
        "Mutual Fund" => &["Investmentfonds", "Fondo de inversión", "Fonds commun de placement", "Fondo comune di investimento", "Fundo de investimento", "Beleggingsfonds", "投資信託", "뮤추얼 펀드", "共同基金", "共同基金", "Паевой фонд"],
        "Index" => &["Index", "Índice", "Indice", "Indice", "Índice", "Index", "指数", "지수", "指数", "指數", "Индекс"],
        "Currency" => &["Währung", "Divisa", "Devise", "Valuta", "Moeda", "Valuta", "通貨", "통화", "货币", "貨幣", "Валюта"],
        "Cryptocurrency" => &["Kryptowährung", "Criptomoneda", "Cryptomonnaie", "Criptovaluta", "Criptomoeda", "Cryptovaluta", "暗号資産", "암호화폐", "加密货币", "加密貨幣", "Криптовалюта"],
        "Futures" => &["Futures", "Futuros", "Contrats à terme", "Futures", "Futuros", "Futures", "先物", "선물", "期货", "期貨", "Фьючерсы"],
        "Option" => &["Option", "Opción", "Option", "Opzione", "Opção", "Optie", "オプション", "옵션", "期权", "期權", "Опцион"],
        // ----- Common officer titles -----
        "Chief Executive Officer" => &["Vorstandsvorsitzender", "Director ejecutivo", "Directeur général", "Amministratore delegato", "Diretor executivo", "Algemeen directeur", "最高経営責任者", "최고경영자", "首席执行官", "執行長", "Генеральный директор"],
        "Chief Financial Officer" => &["Finanzvorstand", "Director financiero", "Directeur financier", "Direttore finanziario", "Diretor financeiro", "Financieel directeur", "最高財務責任者", "최고재무책임자", "首席财务官", "財務長", "Финансовый директор"],
        "Chief Operating Officer" => &["Operativer Geschäftsführer", "Director de operaciones", "Directeur des opérations", "Direttore operativo", "Diretor de operações", "Operationeel directeur", "最高執行責任者", "최고운영책임자", "首席运营官", "營運長", "Операционный директор"],
        "Chief Technology Officer" => &["Technischer Direktor", "Director de tecnología", "Directeur de la technologie", "Direttore tecnico", "Diretor de tecnologia", "Technisch directeur", "最高技術責任者", "최고기술책임자", "首席技术官", "技術長", "Технический директор"],
        "Chairman" => &["Vorsitzender", "Presidente", "Président", "Presidente", "Presidente", "Voorzitter", "会長", "회장", "董事长", "董事長", "Председатель"],
        "Chairman of the Board" => &["Aufsichtsratsvorsitzender", "Presidente del consejo", "Président du conseil d'administration", "Presidente del consiglio di amministrazione", "Presidente do conselho", "Voorzitter van de raad van bestuur", "取締役会長", "이사회 의장", "董事会主席", "董事會主席", "Председатель совета директоров"],
        "President" => &["Präsident", "Presidente", "Président", "Presidente", "Presidente", "President", "社長", "사장", "总裁", "總裁", "Президент"],
        "Director" => &["Direktor", "Director", "Administrateur", "Consigliere", "Diretor", "Directeur", "取締役", "이사", "董事", "董事", "Директор"],
        "Independent Director" => &["Unabhängiger Direktor", "Director independiente", "Administrateur indépendant", "Consigliere indipendente", "Diretor independente", "Onafhankelijk directeur", "独立取締役", "사외이사", "独立董事", "獨立董事", "Независимый директор"],
        "Founder" => &["Gründer", "Fundador", "Fondateur", "Fondatore", "Fundador", "Oprichter", "創業者", "창립자", "创始人", "創辦人", "Основатель"],
        "Co-Founder" => &["Mitgründer", "Cofundador", "Cofondateur", "Cofondatore", "Cofundador", "Medeoprichter", "共同創業者", "공동 창립자", "联合创始人", "共同創辦人", "Сооснователь"],
        "General Counsel" => &["Chefjustiziar", "Asesor jurídico general", "Directeur juridique", "Direttore legale", "Diretor jurídico", "Hoofd juridische zaken", "法務責任者", "법무총괄", "总法律顾问", "總法律顧問", "Главный юрисконсульт"],
        "Secretary" => &["Sekretär", "Secretario", "Secrétaire", "Segretario", "Secretário", "Secretaris", "秘書役", "총무", "秘书", "秘書", "Секретарь"],
        "Treasurer" => &["Schatzmeister", "Tesorero", "Trésorier", "Tesoriere", "Tesoureiro", "Penningmeester", "財務担当役員", "재무 담당자", "司库", "司庫", "Казначей"],
        "Vice President" => &["Vizepräsident", "Vicepresidente", "Vice-président", "Vicepresidente", "Vice-presidente", "Vicepresident", "副社長", "부사장", "副总裁", "副總裁", "Вице-президент"],
        "Senior Vice President" => &["Senior-Vizepräsident", "Vicepresidente sénior", "Vice-président principal", "Vicepresidente senior", "Vice-presidente sênior", "Senior vicepresident", "上級副社長", "수석 부사장", "高级副总裁", "資深副總裁", "Старший вице-президент"],
        "Executive Vice President" => &["Geschäftsführender Vizepräsident", "Vicepresidente ejecutivo", "Vice-président exécutif", "Vicepresidente esecutivo", "Vice-presidente executivo", "Uitvoerend vicepresident", "執行副社長", "전무", "执行副总裁", "執行副總裁", "Исполнительный вице-президент"],
        _ => return None,
    };
    Some(row[lang as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_lookup() {
        assert_eq!(lookup(DictLang::Ja, "Technology"), Some("テクノロジー"));
        assert_eq!(lookup(DictLang::De, "Healthcare"), Some("Gesundheitswesen"));
        assert_eq!(lookup(DictLang::ZhHant, "Real Estate"), Some("房地產"));
    }

    #[test]
    fn officer_title_lookup() {
        assert_eq!(
            lookup(DictLang::Ja, "Chief Executive Officer"),
            Some("最高経営責任者")
        );
    }

    #[test]
    fn unknown_term_misses() {
        assert_eq!(lookup(DictLang::De, "Apple Inc."), None);
        assert_eq!(lookup(DictLang::De, "technology"), None);
    }

    #[test]
    fn dict_lang_resolution() {
        let ja = Lang::parse("ja-JP").unwrap();
        assert_eq!(DictLang::from_lang(&ja), Some(DictLang::Ja));
        let tw = Lang::parse("zh-TW").unwrap();
        assert_eq!(DictLang::from_lang(&tw), Some(DictLang::ZhHant));
        let cn = Lang::parse("zh-CN").unwrap();
        assert_eq!(DictLang::from_lang(&cn), Some(DictLang::ZhHans));
        let th = Lang::parse("th").unwrap();
        assert_eq!(DictLang::from_lang(&th), None);
    }
}
