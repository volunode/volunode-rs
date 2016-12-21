extern crate treexml;

//use std::convert::TryFrom;

use errors;
use util;

#[derive(Clone, Debug, Default)]
pub struct HostInfo {
    pub tz_shift: i64,
    pub domain_name: String,
    pub serialnum: String,
    pub ip_addr: String,
    pub host_cpid: String,

    pub p_ncpus: i64,
    pub p_vendor: String,
    pub p_model: String,
    pub p_features: String,
    pub p_fpops: f64,
    pub p_iops: f64,
    pub p_membw: f64,
    pub p_calculated: f64,
    pub p_vm_extensions_disabled: bool,

    pub m_nbytes: f64,
    pub m_cache: f64,
    pub m_swap: f64,

    pub d_total: f64,
    pub d_free: f64,

    pub os_name: String,
    pub os_version: String,
    pub product_name: String,

    pub mac_address: String,
}

impl<'a> From<&'a HostInfo> for treexml::ElementBuilder {
    fn from(v: &HostInfo) -> treexml::ElementBuilder {
        let mut e = treexml::ElementBuilder::new("host_info");
        e.children(vec![
            &mut util::serialize_node("tz_shift", &v.tz_shift),
            &mut util::serialize_node("domain_name", &v.domain_name),
            &mut util::serialize_node("serialnum", &v.serialnum),
            &mut util::serialize_node("ip_addr", &v.ip_addr),
            &mut util::serialize_node("host_cpid", &v.host_cpid),
            &mut util::serialize_node("p_ncpus", &v.p_ncpus),
            &mut util::serialize_node("p_vendor", &v.p_vendor),
            &mut util::serialize_node("p_model", &v.p_model),
            &mut util::serialize_node("p_features", &v.p_features),
            &mut util::serialize_node("p_fpops", &v.p_fpops),
            &mut util::serialize_node("p_iops", &v.p_iops),
            &mut util::serialize_node("p_membw", &v.p_membw),
            &mut util::serialize_node(
                "p_calculated",
                &v.p_calculated
            ),
            &mut util::serialize_node(
                "p_vm_extensions_disabled",
                &v.p_vm_extensions_disabled
            ),
            &mut util::serialize_node("m_nbytes", &v.m_nbytes),
            &mut util::serialize_node("m_cache", &v.m_cache),
            &mut util::serialize_node("m_swap", &v.m_swap),
            &mut util::serialize_node("d_total", &v.d_total),
            &mut util::serialize_node("d_free", &v.d_free),
            &mut util::serialize_node("os_name", &v.os_name),
            &mut util::serialize_node("os_version", &v.os_version),
            &mut util::serialize_node(
                "product_name",
                &v.product_name
            ),
            &mut util::serialize_node("mac_address", &v.mac_address),
        ]);
        e
    }
}

//impl<'a> TryFrom<&'a treexml::Element> for HostInfo {
//    type Err = errors::Error;
fn try_from(node: &treexml::Element) -> Result<HostInfo, errors::Error> {
    let mut e = HostInfo::default();
    for ref n in &node.children {
        util::deserialize_node("p_fpops", &n, &mut e.p_fpops)?;
        util::deserialize_node("p_iops", &n, &mut e.p_iops)?;
        util::deserialize_node("p_membw", &n, &mut e.p_membw)?;
        util::deserialize_node("p_calculated", &n, &mut e.p_calculated)?;
        util::deserialize_node(
            "p_vm_extensions_disabled",
            &n,
            &mut e.p_vm_extensions_disabled,
        )?;
        util::deserialize_node("host_cpid", &n, &mut e.host_cpid)?;
        util::deserialize_node("product_name", &n, &mut e.product_name)?;
        util::deserialize_node("mac_address", &n, &mut e.mac_address)?;
        util::deserialize_node("domain_name", &n, &mut e.domain_name)?;
        util::deserialize_node("ip_addr", &n, &mut e.ip_addr)?;
        util::deserialize_node("p_vendor", &n, &mut e.p_vendor)?;
        util::deserialize_node("p_model", &n, &mut e.p_model)?;
        util::deserialize_node("os_name", &n, &mut e.os_name)?;
        util::deserialize_node("os_version", &n, &mut e.os_version)?;
        util::deserialize_node("p_features", &n, &mut e.p_features)?;
        util::deserialize_node("timezone", &n, &mut e.tz_shift)?;
        util::deserialize_node("p_ncpus", &n, &mut e.p_ncpus)?;
        util::deserialize_node("m_nbytes", &n, &mut e.m_nbytes)?;
        util::deserialize_node("m_cache", &n, &mut e.m_cache)?;
        util::deserialize_node("m_swap", &n, &mut e.m_swap)?;
        util::deserialize_node("d_total", &n, &mut e.d_total)?;
        util::deserialize_node("d_free", &n, &mut e.d_free)?;
    }
    Ok(e)
}
//}
