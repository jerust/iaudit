use regex::Regex;

/// 返回所有匹配到的字符串的起始位置及其文本内容
pub fn match_string_with_offset(regex: &Regex, haystack: &str) -> Vec<(usize, String)> {
    regex
        .find_iter(haystack)
        .map(|mat| (mat.start(), mat.as_str().into()))
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn mock_match_string_with_offset() {
        let document = "
第一章 总则
第一条 为规范公司货物、工程和服务等采购工作的管理制度及流程，根据国家相关法律、法规，结合公司业务发展需求，保证采购管理健康有序的开展，特制定本办法。
第二条 公司相关采购活动按照本办法执行，公司广告及宣传业务相关采购活动管理办法另行制定。
第二章 采购的分类
第三条 根据公司业务发展要求和特点，公司采购类别主要分为货物、工程和服务三类采购。
第四条 货物类的采购分为网络设备类、通用设备类、通用软件类、交通工具类、礼品类、耗材类、印刷品类等。
第五条 工程类的采购包含机房、管道和光缆线路等基础建设工程、系统集成建设工程、有线配套工程、综合线网工程、终端设备安装工程及装修类工程等。
第六条 服务类的采购包含新技术研发服务类、维保服务类、工程设计及监理类、咨询类、租赁服务类（带宽租用、房屋租用等）等（除广告服务类外）。
第七条 物资采购以集中（框架）采购和专项（单次）采购为主，零星采购为辅。原则上按年度、季度及月度实施计划管理。
";
        let regex = regex::Regex::new(r"第(?:[零一二三四五六七八九十百千万]*\d*)条").unwrap();
        super::match_string_with_offset(&regex, document)
            .into_iter()
            .for_each(|(offset, string)| println!("{}: {}", offset, string));
    }
}
